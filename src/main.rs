use axum::{
    routing::{get,post, delete},
    Router,
    extract::{Extension, Path, State},
    Json,
    middleware::{self, from_fn, Next},
    http::{Request, StatusCode, Response},
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

use axum::body::Body;

async fn my_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    println!("→ incoming request: {}", req.uri());
    
    let res = next.run(req).await;

    println!("← outgoing response: {}", res.status());
    Ok(res)
}


#[tokio::main]
async fn main() {
    // build our application with a single route
    let db = init_db().await;

    let app = Router::new()
    .merge(post_routes())
    .layer(from_fn(my_middleware)) // <-- 여기 미들웨어 추가
    .layer(CorsLayer::permissive())
    .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}