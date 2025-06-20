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
use redis::AsyncCommands;
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

    // Redis에 캐시된 응답이 있는지 확인
    match conn.get::<_, Option<Vec<u8>>>(&key).await {
        Ok(Some(cached_body)) => {
            println!("🔄 Redis cache hit for {}", key);
            let response = Response::builder()
                .status(200)
                .header("X-Cache", "HIT")
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


#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = init_db().await;
    let redis_client = Arc::new(redis::Client::open("redis://127.0.0.1/").unwrap());

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}