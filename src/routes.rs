// src/routes.rs

use axum::{
    extract::{State, Json, Path},
    routing::{get, post, delete, put},
    Router,
    http::StatusCode,
};
use sqlx::{SqlitePool};
use crate::models::{Post, CreatePost, UpdatePost, PostResponse,
                    Comment, CreateComment,
                    User, CreateUser};

pub fn post_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/comments", get(list_comments).post(create_comment))
        .route("/accounts", get(list_accounts).post(create_account))
}


pub fn post_routes_cache() -> Router<SqlitePool> {
    Router::new()
        .route("/posts/:id", get(get_posts).delete(delete_post).put(update_post))
}


async fn create_post(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, (StatusCode, String)> {

    let mut tx = db.begin().await.map_err(internal_error)?;
    sqlx::query("INSERT INTO posts (title, content, user_id) VALUES (?, ?, ?)")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(&payload.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("Error inserting post: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create post".to_string())
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

async fn list_posts(State(db): State<SqlitePool>) -> Json<Vec<Post>> {
    let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts")
        .fetch_all(&db)
        .await
        .unwrap();

//TODO : unwarp 사용 안하는게 좋음
    Json(posts)
}
async fn get_posts(
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
        .bind(id)
        .fetch_one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    
    /* related post 가져오기기 */
    let all_posts: Vec<Post> = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_id = ? AND id != ?"
        )
        .bind(post.user_id)
        .bind(post.id)
        .fetch_all(&db)
        .await
        .map_err(|e| {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch related posts".to_string())
        })?;
    
    // 앞 3개만 안전하게 수집
    let related_posts: Vec<Post> = all_posts
        .into_iter()
        .take(3)
        .collect();
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
) -> Result<(), (StatusCode, String)> {

    sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id)
        .execute(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;


    Ok(())
}

async fn update_post(
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) -> Result<Json<Post>, (StatusCode, String)> {

    let mut tx = db.begin().await.map_err(internal_error)?;

    sqlx::query("UPDATE posts SET title = ?, content = ? WHERE id = ?")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(post))
}


async fn create_comment(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateComment>,
) -> Result<Json<Comment>, (StatusCode, String)> {
    
    let mut tx = db.begin().await.map_err(internal_error)?;
    sqlx::query("INSERT INTO comments (content, post_id, user_id) VALUES (?, ?, ?)")
        .bind(&payload.content)
        .bind(&payload.post_id)
        .bind(&payload.user_id)
        .execute(&mut *tx)
        .await
        .unwrap();

    // 최근 삽입된 row의 id 가져오기
    let last_id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    // 해당 id로 다시 조회
    let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
        .bind(last_id.0)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(comment))
}
async fn list_comments(State(db): State<SqlitePool>) -> Json<Vec<Comment>> {
    let comments = sqlx::query_as::<_, Comment>("SELECT * FROM comments")
        .fetch_all(&db)
        .await
        .unwrap();

    Json(comments)
}
async fn create_account(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;

    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(&payload.username)
        .bind(&payload.password)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    let last_id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(last_id.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(user))
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("에러: {}", e))
}

async fn list_accounts(State(db): State<SqlitePool>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&db)
        .await
        .unwrap();

    Json(users)
}