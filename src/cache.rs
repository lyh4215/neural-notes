//Write-Behind Redis Cache
use axum::{
    extract::{Path, State}, http::{response, Request, Response, StatusCode}, middleware::Next, Json
};
use sqlx::{
    SqlitePool,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use redis::{AsyncCommands, RedisResult};
use axum::http::Method;
use http_body_util::BodyExt;
use bytes::Bytes;
use axum::body::Body;
use colored::*;
use futures_util::StreamExt;

pub async fn init_cache() ->  redis::Client {
    //redis setting (keyevent notification channel)
    let redis_client =redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = redis_client.get_connection().unwrap();
    let _: () = redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg("Ex")
        .query(&mut conn).unwrap();
    redis_client
}
//background worker
pub async fn write_behind(client: redis::Client, db: SqlitePool) {
    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            panic!("write behind thread start failed");
        }
    };
    loop {
        // Redis에서 post:* 키들을 스캔
        let keys: Vec<String> = match conn.keys("dirty:/posts/*").await {
            Ok(k) => k,
            Err(e) => {
                eprintln!("❌ Failed to get keys: {e}");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        for key in keys {
            println!("key : {key}");
            if let Ok(Some(bytes)) = (conn.get::<_, Option<Vec<u8>>>(&key).await) {
                // 여기서 실제 DB 저장 로직 호출 (예시로 println 사용)
                //let strings = String::from_utf8_lossy(&bytes);
                //println!("💾 Writing to DB from key: {key} → {strings}");
                use crate::models::PostResponse;
                let json: PostResponse = serde_json::from_slice(&bytes).unwrap();
                use crate::models::UpdatePost;
                use crate::posts::__update_post_from_cache;
                //TODO : from PostResponse, to UpdatePost
                let update_json  = UpdatePost {
                    title : Some(json.title),
                    content : Some(json.content),
                };
                let _ = __update_post_from_cache(State(db.clone()), Path(json.id), Json(update_json)).await;

                //key 제거
                let _: () = conn.del(&key).await.unwrap_or(());
                let clean_key = key.strip_prefix("dirty:").unwrap_or(&key).to_string();
                let _: () = conn.set_ex(&clean_key, bytes, 10).await.unwrap_or(()); // 예: 1시간 만료
                println!("Write behind for : {key}");
            }


        }

        // 10초마다 반복
        sleep(Duration::from_secs(10)).await;
    }
}


pub async fn delete_event_listener(
    client: redis::Client,
    db: SqlitePool,
)  {
    let mut pubsub_conn = client.get_async_pubsub().await.unwrap();

    // expire 이벤트 구독
    pubsub_conn.subscribe("__keyevent@0__:expired").await.unwrap();

    println!("{} Redis expired event listening", "Start".green().bold());

    while let Some(msg) = pubsub_conn.on_message().next().await {
        let expired_key: String = msg.get_payload().unwrap();

        // delete:/posts/ 만 감지
        if let Some(post_id_str) = expired_key.strip_prefix("delete:/posts/") {
            if let Ok(post_id) = post_id_str.parse::<i64>() {
                println!("🧹 expired 감지됨: delete marker for post_id={}", post_id);

                // 실제 DB에서 삭제
                let result = sqlx::query("DELETE FROM posts WHERE id = ?")
                    .bind(post_id)
                    .execute(&db)
                    .await.unwrap();

                println!("✅ DB에서 post {} 삭제 완료 ({} rows affected)", post_id, result.rows_affected());
            }
        }
    }
}
//middleware
pub async fn middleware_cache(
    State(redis_client): State<redis::Client>,
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


    // 이미 삭제된거면, 안보이게 해야 함 
    let del_key = String::from("delete:") + &key;
    if conn.exists(&del_key).await.unwrap() {
        let final_response = Response::builder()
            .status(404)
            .body(Body::empty())
            .unwrap();

        return Ok(final_response);
    }

    //삭제 안됐을때.
    
    //get 요청
    match req.method() {
        &Method::GET => {
            println!("GET");
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
                    println!("❌ Cache miss, find in dirty");
                    let dirty = String::from("dirty:") + &key;
                    match conn.get::<_, Option<Vec<u8>>>(&dirty).await {
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
                            println!("Cache Missed again");
                        }
                        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                    }
                    // 계속 진행
                }
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        &Method::PUT => {
            println!("PUT");
            use crate::models::UpdatePost;
            use crate::models::PostResponse;
            let dirty_key = String::from("dirty:") + &key;
            match conn.get::<_, Option<Vec<u8>>>(&dirty_key).await {
                Ok(Some(cached_body)) => {
                    println!("✅ Redis dirty hit {}", key);
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
                    let dirty_key = String::from("dirty:") + &key;
                    conn.set(dirty_key, cloned_bytes).await.unwrap_or(());
                    let _ :RedisResult<i32>  = conn.del(key).await;
                    
                    let final_response = Response::builder()
                        .status(200)
                        .header("X-Cache", "HIT")
                        .header("Content-Type", "application/json")
                        .body(Body::from(response_bytes))
                        .unwrap();

                    return Ok(final_response);
                }
                Ok(None) => {
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
                            let dirty_key = String::from("dirty:") + &key;
                            conn.set(dirty_key, cloned_bytes).await.unwrap_or(());
                            let _ :RedisResult<i32>  = conn.del(key).await;
                            
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
                }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            
            }
            
            // write-back 캐시 등 수행
        },
        &Method::DELETE => { //DELETE일때는 캐시에 delete tag 붙여서 기록
            println!("DEL");
            let result :RedisResult<i32> = conn.del(&key).await;
            match result {
                Ok(i)   => println!("deleted {i}"),
                Err(e) => println!("err {e}"),
            }
            let dirty_key = String::from("dirty:") + &key;
            let _  :RedisResult<i32>= conn.del(&dirty_key).await;
            let del_key = String::from("delete:") + &key;
            let _ : RedisResult<()>= conn.set_ex(&del_key, "1", 10).await;
            let response = Response::builder()
                .status(204)
                .body(axum::body::Body::empty())
                .unwrap();
            return Ok(response);
        }
        _ => {
            println!("🔴 기타 요청");
        }
    }

    let method = req.method().clone();
    // 없으면 요청을 처리
    let response = next.run(req).await;

    //GET : Cache miss
    //PUT : Cache miss
    //DELETE : not reached
    match method {
        Method::GET | Method::PUT => {
            // 바디 추출
            let (parts, body) = response.into_parts();
            let collected = body.collect().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let bytes: Bytes = collected.to_bytes(); // bytes로 변환
            let cloned_body = bytes.clone();

            // Redis에 저장 (TTL: 60초)
            let _: () = conn.set_ex::<_, _, ()>(key, cloned_body.to_vec(), 60).await.unwrap_or(());

            //response 재조립립
            let final_response = Response::from_parts(parts, Body::from(bytes));
            Ok(final_response)
        },
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }

}
