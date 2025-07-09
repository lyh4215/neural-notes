// src/posts.rs

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    middleware::{self, Next, from_fn},
    routing::{delete, get, post, put},
};

use jwt_authorizer::JwtClaims;

use crate::{
    auth::UserClaims,
    models::{CreatePost, Post, PostResponse, UpdatePost, User},
};
use redis::{Client, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, pool::Pool};

//cache
use axum_redis_cache::{CacheManager, CacheState};

//embedding
use pgvector::Vector;

pub fn routes(
    //mut conn: MultiplexedConnection
    cache_manager: CacheManager,
) -> Router<Pool<Postgres>> {
    let stat = cache_manager.get_state();
    Router::new()
        .merge(post_routes_auth())
        .merge(post_routes_cache().layer(middleware::from_fn_with_state(
            stat.clone(),
            axum_redis_cache::middleware,
        )))
}

fn post_routes_auth() -> Router<Pool<Postgres>> {
    Router::new().route("/posts", get(list_posts).post(create_post))
}

fn post_routes_cache() -> Router<Pool<Postgres>> {
    Router::new().route(
        "/posts/:id",
        get(get_posts).delete(delete_post).put(update_post),
    )
}

pub fn write_to_cache(old: String, new: String) -> String {
    let parsed_body: UpdatePost = serde_json::from_str(&new).unwrap();
    let mut payload: PostResponse = serde_json::from_str(&old).unwrap();

    if let Some(content) = parsed_body.content {
        payload.content = content;
    }
    if let Some(title) = parsed_body.title {
        payload.title = title;
    }
    serde_json::to_string(&payload).unwrap()
}

pub async fn callback(db: Pool<Postgres>, value: String) {
    use crate::models::PostResponse;
    let json: PostResponse = serde_json::from_str(&value).unwrap();
    use crate::models::UpdatePost;
    use crate::posts::__update_post_from_cache;
    //TODO : from PostResponse, to UpdatePost
    let update_json = UpdatePost {
        title: Some(json.title),
        content: Some(json.content),
    };
    let _ = __update_post_from_cache(State(db), Path(json.id), Json(update_json)).await;
}

pub async fn delete_callback(db: Pool<Postgres>, key: String) {
    if let Ok(post_id) = key.parse::<i64>() {
        println!("🧹 expired 감지됨: delete marker for post_id={}", post_id);

        // 실제 DB에서 삭제
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(post_id)
            .execute(&db)
            .await
            .unwrap();

        println!(
            "✅ DB에서 post {} 삭제 완료 ({} rows affected)",
            post_id,
            result.rows_affected()
        );
    }
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

async fn create_post(
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

async fn list_posts(
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

    //TODO : unwarp 사용 안하는게 좋음
    Json(posts)
}
async fn get_posts(
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
async fn delete_post(
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

//todo : make this
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
    let embed_resp = client
        .post("http://embed_api:8001/embed")
        .json(&serde_json::json!({ "text": payload.content })) //TODO : modify this
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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

async fn get_related_post(post: &Post, db: &Pool<Postgres>) -> Vec<Post> {
    /* related post 가져오기기 */
    let Some(embedding) = &post.embedding else {
        return Vec::new();
    };
    // 유사도 기반으로 관련 포스트 3개 반환
    let related_posts = sqlx::query_as::<_, Post>(
        r#"
        SELECT * FROM posts
        WHERE user_id = $1
        AND id != $2
        AND embedding IS NOT NULL
        ORDER BY embedding <-> $3
        LIMIT 3
        "#,
    )
    .bind(post.user_id)
    .bind(post.id)
    .bind(embedding)
    .fetch_all(db)
    .await
    .unwrap_or_else(|e| {
        eprintln!("DB error during related post fetch: {}", e);
        Vec::new()
    });

    related_posts
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("에러: {}", e))
}
