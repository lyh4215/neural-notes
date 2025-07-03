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
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

//cache
use axum_redis_cache::{CacheManager, CacheState};

pub fn routes(
    //mut conn: MultiplexedConnection
    cache_manager: CacheManager,
) -> Router<SqlitePool> {
    let stat = cache_manager.get_state();
    Router::new()
        .merge(post_routes_auth())
        .merge(post_routes_cache().layer(middleware::from_fn_with_state(
            stat.clone(),
            axum_redis_cache::middleware,
        )))
}

fn post_routes_auth() -> Router<SqlitePool> {
    Router::new().route("/posts", get(list_posts).post(create_post))
}

fn post_routes_cache() -> Router<SqlitePool> {
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

pub async fn callback(db: SqlitePool, value: String) {
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

pub async fn delete_callback(db: SqlitePool, key: String) {
    if let Ok(post_id) = key.parse::<i64>() {
        println!("🧹 expired 감지됨: delete marker for post_id={}", post_id);

        // 실제 DB에서 삭제
        let result = sqlx::query("DELETE FROM posts WHERE id = ?")
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

async fn create_post(
    State(db): State<SqlitePool>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;
    sqlx::query("INSERT INTO posts (title, content, user_id) VALUES (?, ?, ?)")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(&user.sub)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Error inserting post: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create post".to_string(),
            )
        })?;

    // 최근 삽입된 row의 id 가져오기
    let last_id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    // 해당 id로 다시 조회
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
        .bind(last_id.0)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(post))
}

async fn list_posts(
    State(db): State<SqlitePool>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Json<Vec<Post>> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_id = ? 
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
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ? AND user_id = ?")
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
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query("DELETE FROM posts WHERE id = ? and user_id = ?")
        .bind(id)
        .bind(user.sub)
        .execute(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    Ok(())
}

//todo : make this
pub async fn update_post(
    State(db): State<SqlitePool>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;

    sqlx::query("UPDATE posts SET title = ?, content = ? WHERE id = ? AND user_id = ?")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(id)
        .bind(user.sub)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
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
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) {
    let mut tx = db.begin().await.map_err(internal_error).unwrap();

    sqlx::query("UPDATE posts SET title = ?, content = ? WHERE id = ?")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))
        .unwrap();

    tx.commit().await.map_err(internal_error).unwrap();
}

async fn get_related_post(post: &Post, db: &SqlitePool) -> Vec<Post> {
    /* related post 가져오기기 */
    let all_posts: Vec<Post> =
        sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE user_id = ? AND id != ?")
            .bind(post.user_id)
            .bind(post.id)
            .fetch_all(db)
            .await
            .map_err(|e| {
                eprintln!("DB error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch related posts".to_string(),
                )
            })
            .unwrap();

    // 앞 3개만 안전하게 수집
    let related_posts: Vec<Post> = all_posts.into_iter().take(3).collect();

    related_posts
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("에러: {}", e))
}
