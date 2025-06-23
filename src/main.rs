use axum::{
    routing::{get,post, delete},
    Router,
    extract::{Extension, Path, State},
    Json,
    middleware::{self, from_fn, Next},
    http::{Request, StatusCode, Response},
};

use sqlx::{
    SqlitePool,
    FromRow,
};

use tower_http::cors::{CorsLayer, Any};
use redis::{AsyncCommands, UpdateCheck};
use redis::aio::PubSub;
use axum::http::Method;
use futures_util::stream::StreamExt; // ✅ .next() 사용 가능하게 함
use http_body_util::BodyExt;

use std::sync::Arc;
mod db;
mod models;
mod routes;
use routes::{post_routes, post_routes_cache};
use models::{Post, CreatePost};
use db::init_db;
use axum::body::Body;
use bytes::Bytes;

use crate::models::UpdatePost;

async fn middleware_cache(
    State(redis_client): State<Arc<redis::Client>>,
    req: Request<Body>,
    next: Next
) -> Result<Response<Body>, StatusCode> {
    let key = req
    .uri()
    .path_and_query()
    .map(|pq| pq.as_str().to_string()) // 👈 복사
    .unwrap_or_else(|| "".to_string());

    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    //get 요청
    match req.method() {
        &Method::GET => {
            // 캐시 로직 등 수행
            // Redis에 캐시된 응답이 있는지 확인
            match conn.get::<_, Option<Vec<u8>>>(&key).await {
                Ok(Some(cached_body)) => {
                    println!("🔄 Redis cache hit for {}", key);
                    let response = Response::builder()
                        .status(200)
                        .header("X-Cache", "HIT")
                        .header("Content-Type", "application/json")
                        .body(Body::from(cached_body))
                        .unwrap();
                    return Ok(response);
                }
                Ok(None) => {
                    println!("❌ Cache miss");
                    // 계속 진행
                }
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        &Method::PUT => {
            use models::UpdatePost;
            use models::PostResponse;
            match conn.get::<_, Option<Vec<u8>>>(&key).await {
                Ok(Some(cached_body)) => {
                    println!("✅ Redis cache hit for {}", key);
                    let mut payload: PostResponse = serde_json::from_slice(&cached_body).unwrap();

                    let (parts, body ) = req.into_parts(); //consume
                    let collected = body.collect().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    let bytes: Bytes = collected.to_bytes(); // bytes로 변환
                    let parsed_body: UpdatePost = serde_json::from_slice(&bytes).unwrap();

                    if let Some(content) = parsed_body.content {
                        payload.content = content;
                    }
                    if let Some(title) = parsed_body.title {
                        payload.title = title;
                    }
                    let response_json = serde_json::to_string(&payload).unwrap();
                    let response_bytes = response_json.into_bytes();
                    let cloned_bytes = response_bytes.clone();
                    conn.set_ex(key, cloned_bytes, 60).await.unwrap_or(());
                    
                    let final_response = Response::builder()
                        .status(200)
                        .header("X-Cache", "HIT")
                        .header("Content-Type", "application/json")
                        .body(Body::from(response_bytes))
                        .unwrap();

                    return Ok(final_response);
                }
                Ok(None) => {
                    println!("❌ Cache miss");
                    // 계속 진행
                }
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
            
            // write-back 캐시 등 수행
        },
        _ => {
            println!("🔴 기타 요청");
        }
    }


    // 없으면 요청을 처리
    let response = next.run(req).await;

    // 바디 추출
    let (parts, body) = response.into_parts();
    let collected = body.collect().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let bytes: Bytes = collected.to_bytes(); // bytes로 변환
    let cloned_body = bytes.clone();

    // Redis에 저장 (TTL: 60초)
    let _: () = conn.set_ex::<_, _, ()>(key, cloned_body.to_vec(), 60).await.unwrap_or(());

    // 다시 Response로 재조립
    let final_response = Response::from_parts(parts, Body::from(bytes));
    Ok(final_response)
}

async fn middleware_logger(    
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    println!("→ incoming request: {}", req.uri());

    
    let res = next.run(req).await;

    println!("← outgoing response: {}", res.status());
    Ok(res)

}

async fn start_write_back_listener(client: Arc<redis::Client>) {
    let mut pubsub = client.get_async_pubsub().await.expect("Failed to get pubsub");
    pubsub.subscribe("__keyevent@0__:expired").await.unwrap();

    let mut stream = pubsub.into_on_message();

    while let Some(msg) = stream.next().await {
        let key: String = msg.get_payload().unwrap();
        println!("Key expired: {}", key);

        // TODO: flush to DB
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = init_db().await;
    let redis_client = Arc::new(redis::Client::open("redis://127.0.0.1/").unwrap());


    //redis setting (keyevent notification channel)
    let mut conn = redis_client.get_connection().unwrap();
    let _: () = redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg("Ex")
        .query(&mut conn)
        .unwrap();

    let posts_routes = post_routes();
    let posts_cache_routes = post_routes_cache()
        .layer(middleware::from_fn_with_state(
            Arc::clone(&redis_client),
            middleware_cache,
        ));

    let app = Router::new()
        .merge(posts_routes)
        .merge(posts_cache_routes)
        .layer(from_fn(middleware_logger))
        .layer(CorsLayer::permissive())
        .with_state(db);

    // for write-back handling
    tokio::spawn(start_write_back_listener(Arc::clone(&redis_client)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}