// src/routes.rs

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};

use jwt_authorizer::JwtClaims;

use crate::{
    auth::UserClaims,
    models::{
        Comment, CreateComment, CreatePost, CreateUser, Post, PostResponse, UpdatePost, User,
    },
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub fn public_routes() -> Router<SqlitePool> {
    Router::new().route("/accounts", get(list_accounts).post(create_account))
}

pub fn routes() -> Router<SqlitePool> {
    Router::new().route("/comments", get(list_comments).post(create_comment))
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

use bcrypt::{DEFAULT_COST, hash};

async fn create_account(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;

    let hashed_password = hash(&payload.password, DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("hash error: {e}"),
        )
    })?;

    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(&payload.username)
        .bind(&hashed_password)
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
