use axum::{
    routing::{get,post, delete},
    Router,
    extract::{Extension, Path, State},
    Json
};

use sqlx::{
    SqlitePool,
    FromRow,
};

use tower_http::cors::{CorsLayer, Any};


mod db;
mod models;
mod routes;
use routes::post_routes;
use models::{Post, CreatePost};
use db::init_db;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = init_db().await;

    let app = Router::new()
    .merge(post_routes())
    .layer(CorsLayer::permissive())
    .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}