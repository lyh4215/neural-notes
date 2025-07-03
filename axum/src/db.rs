// src/db.rs

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub async fn init_db() -> SqlitePool {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./posts.db".to_string());
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5) // Set the maximum number of connections
        .connect_with(options)
        .await
        .expect("DB connection failed");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Database migration failed");
    db
}
