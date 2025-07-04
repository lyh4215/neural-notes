// src/db.rs

use sqlx::{
    SqlitePool,
    postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode},
    sqlite::SqliteConnectOptions,
};

use std::time::Duration;

pub async fn init_db() -> PgPool {
    let host = std::env::var("DATABASE_HOST").expect("DATABASE_HOST not set");
    let user = std::env::var("DATABASE_USER").expect("DATABASE_USER not set");
    let port = std::env::var("DATABASE_PORT")
        .unwrap_or("5432".to_string())
        .parse::<u16>()
        .expect("DATABASE_PORT must be a valid u16");
    let password = std::env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not set");
    let database = std::env::var("DATABASE_NAME").expect("DATABASE_NAME not set");

    let conn = PgConnectOptions::new()
        .host(&host)
        .port(port)
        .username(&user)
        .password(&password)
        .database(&database)
        .ssl_mode(PgSslMode::Disable);

    let db = loop {
        match PgPoolOptions::new()
            .max_connections(5) // 기존 설정값으로 맞추면 됩니다
            .connect_with(conn.clone())
            .await
        {
            Ok(pool) => {
                println!("✅ DB connected successfully.");
                break pool;
            }
            Err(e) => {
                eprintln!("⚠️ DB connection failed, retrying in 3s: {:?}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Database migration failed");

    db
}

pub async fn init_db_sqlite() -> SqlitePool {

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
