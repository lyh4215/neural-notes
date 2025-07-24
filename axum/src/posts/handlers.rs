use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use jwt_authorizer::JwtClaims;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, pool::Pool};

use super::graph::get_related_post;
use super::models::*;
use super::models::{EmbeddingRequest, EmbeddingResponse, QueryRequest};
use super::utils::{
    EmbeddingApiError, call_embedding_api_with_retry, check_embedding_api_health, internal_error,
};
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

pub async fn search_posts(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    println!("Received search query in Axum: {}", search_query.q);

    let embed_api_url =
        std::env::var("EMBED_API_URL").unwrap_or_else(|_| "http://embed_api:8001".to_string());
    let health_check_url = format!("{}/health", embed_api_url);
    let query_embedding_url = format!("{}/query-embedding", embed_api_url);

    // 헬스체크 수행
    check_embedding_api_health(&health_check_url)
        .await
        .map_err(|e| {
            eprintln!("Embedding API health check failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Embedding service unavailable".to_string(),
            )
        })?;

    let embed_data = call_embedding_api_with_retry(
        &query_embedding_url,
        QueryRequest {
            query: search_query.q.clone(),
        },
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to get query embedding from API: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get query embedding".to_string(),
        )
    })?;

    let query_vector = Vector::from(embed_data.embedding);

    let posts = sqlx::query_as::<_, Post>(include_str!("../../sql/search_posts.sql"))
        .bind(query_vector)
        .bind(user.sub)
        .fetch_all(&db)
        .await
        .map_err(|e| {
            eprintln!("Database search failed: {}", e);
            internal_error(e)
        })?;

    Ok(Json(posts))
}

pub async fn __update_post_from_cache(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) {
    let mut tx = db.begin().await.map_err(internal_error).unwrap();

    let embed_api_url =
        std::env::var("EMBED_API_URL").unwrap_or_else(|_| "http://embed_api:8001".to_string());
    let health_check_url = format!("{}/health", embed_api_url);
    let embed_url = format!("{}/embed", embed_api_url);

    let mut vector: Option<Vector> = None;
    // 헬스체크 수행
    if let Err(e) = check_embedding_api_health(&health_check_url).await {
        eprintln!("Embedding API health check failed for update: {}", e);
    } else {
        vector = if let Some(content) = payload.content.clone() {
            let embedding_result =
                call_embedding_api_with_retry(&embed_url, EmbeddingRequest { text: content }).await;

            match embedding_result {
                Ok(data) => {
                    // ➋ pgvector Vector로 변환
                    Some(Vector::from(data.embedding))
                }
                Err(e) => {
                    eprintln!("Failed to get embedding for update, setting to NULL: {}", e);
                    None // Set embedding to NULL
                }
            }
        } else {
            eprintln!("Payload content is None, setting embedding to NULL.");
            None
        };
    }

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
