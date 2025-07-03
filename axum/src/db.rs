// src/db.rs

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

pub async fn init_db() -> SqlitePool {
    let db_path = "posts.db";
    if !Path::new(db_path).exists() {
        println!("DB 파일이 없어 빈 파일을 생성합니다: {}", db_path);
        std::fs::File::create(db_path).expect("DB 파일 생성 실패");
    }

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}", db_path))
        .await
        .expect("DB 연결 실패");
    sqlx::migrate!().run(&db).await.unwrap();
    db
}
