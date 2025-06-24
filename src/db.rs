// src/db.rs

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://posts.db")
        .await
        .expect("DB 연결 실패");
    sqlx::migrate!().run(&db).await.unwrap();
    db
}
