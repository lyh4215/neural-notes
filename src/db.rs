// src/db.rs

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://posts.db")
        .await
        .expect("DB 연결 실패");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(&db)
    .await
    .expect("테이블 생성 실패");
    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS update_timestamp
        AFTER UPDATE ON users
        FOR EACH ROW
        BEGIN
            UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
        END;"
    )
    .execute(&db)
    .await
    .expect("트리거 생성 실패");
    // posts 테이블 생성

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            user_id INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );"
    )
    .execute(&db)
    .await
    .expect("테이블 생성 실패");

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS update_timestamp
        AFTER UPDATE ON posts
        FOR EACH ROW
        BEGIN
            UPDATE posts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
        END;"
    )
    .execute(&db)
    .await
    .expect("트리거 생성 실패");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            post_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );"
    )
    .execute(&db)
    .await
    .expect("테이블 생성 실패");

    db
}
