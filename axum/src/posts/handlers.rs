use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use core::panic;
use jwt_authorizer::JwtClaims;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, pool::Pool};
use std::time::Duration;
use tokio::time::sleep;

use super::graph::get_related_post;
use super::models::{CreatePost, EmbedResponse, Post, PostGraphData, PostResponse, UpdatePost};
use super::utils::internal_error;
use crate::auth::UserClaims;

pub async fn create_post(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;
    let post = sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (title, content, user_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&user.sub)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Error inserting post: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create post".to_string(),
        )
    })?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(post))
}

pub async fn list_posts(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Json<Vec<Post>> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_id = $1 
    ORDER BY created_at DESC;",
    )
    .bind(user.sub)
    .fetch_all(&db)
    .await
    .unwrap();

    //TODO : unwrap 사용 안하는게 좋음
    Json(posts)
}

pub async fn get_posts(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.sub)
        .fetch_one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let related_posts = get_related_post(&post, &db).await;
    let post_response = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at,
        updated_at: post.updated_at,
        user_id: post.user_id,
        related_posts,
    };

    Ok(Json(post_response))
}

pub async fn delete_post(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query("DELETE FROM posts WHERE id = $1 and user_id = $2")
        .bind(id)
        .bind(user.sub)
        .execute(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    Ok(())
}

pub async fn update_post(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;

    sqlx::query("UPDATE posts SET title = $1, content = $2 WHERE id = $3 AND user_id = $4")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(id)
        .bind(user.sub)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    tx.commit().await.map_err(internal_error)?;

    let related_posts = get_related_post(&post, &db).await;

    let post_response: PostResponse = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at,
        updated_at: post.updated_at,
        user_id: post.user_id,
        related_posts,
    };
    Ok(Json(post_response))
}

pub async fn __update_post_from_cache(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) {
    let mut tx = db.begin().await.map_err(internal_error).unwrap();
    let client = reqwest::Client::new();
    let embed_api_url =
        std::env::var("EMBED_API_URL").unwrap_or_else(|_| "http://embed_api:8001".to_string());
    let max_retries = 5;
    let retry_delay = Duration::from_secs(10); // ⏱️ 재시도 간격: 10초

    // Step 1: Wait for /health to return 200 OK
    let mut health_ok = false;
    for attempt in 1..=max_retries {
        match client.get(format!("{}/health", embed_api_url)).send().await {
            Ok(resp) if resp.status().is_success() => {
                health_ok = true;
                break;
            }
            Ok(resp) => {
                eprintln!(
                    "🔁 /health responded with non-200 ({}), attempt {}/{}",
                    resp.status(),
                    attempt,
                    max_retries
                );
            }
            Err(e) => {
                eprintln!("⚠️ Failed to reach /health (attempt {}): {}", attempt, e);
            }
        }
        sleep(retry_delay).await;
    }

    if !health_ok {
        panic!(
            "❌ /health did not return 200 OK after {} attempts. Aborting update.",
            max_retries
        );
    }
    // Step 2: Send the /embed request
    let embed_resp = {
        let mut attempt = 0;
        let mut last_err = None;
        loop {
            attempt += 1;
            match client
                .post(format!("{}/embed", embed_api_url))
                .header(
                    "Authorization",
                    format!(
                        "Bearer {}",
                        std::env::var("EMBED_API_KEY").unwrap_or_default()
                    ),
                )
                .json(&serde_json::json!({ "text": payload.content }))
                .send()
                .await
            {
                Ok(resp) => break Ok(resp),
                Err(e) => {
                    last_err = Some(e.to_string());
                    if attempt >= max_retries {
                        break Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!(
                                "Failed after {} attempts: {}, URL: {}",
                                attempt,
                                last_err.clone().unwrap_or_default(),
                                embed_api_url
                            ),
                        ));
                    }
                    eprintln!(
                        "⚠️ Request failed (attempt {}): {}. Retrying in {}s...",
                        attempt,
                        e,
                        retry_delay.as_secs()
                    );
                    sleep(retry_delay).await;
                }
            }
        }
    }
    .unwrap();

    let embed_resp: EmbedResponse = embed_resp
        .json()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .unwrap();

    // ➋ pgvector Vector로 변환
    let vector = Vector::from(embed_resp.embedding);

    sqlx::query(
        r#"
        UPDATE posts 
        SET title = $1, content = $2 , embedding = $3
        WHERE id = $4
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(vector)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))
    .unwrap();

    tx.commit().await.map_err(internal_error).unwrap();
}
