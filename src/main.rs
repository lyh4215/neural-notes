use axum::{
    Router,
    middleware::{self, from_fn, Next},
    http::{Request, StatusCode, Response},
};

use tower_http::cors::{CorsLayer};

//cache
mod cache;
use cache::{init_cache, write_behind, middleware_cache};


use std::sync::Arc;
mod db;
use db::init_db;

mod models;
mod routes;
use routes::{post_routes, post_routes_cache};

use axum::body::Body;

async fn middleware_logger(    
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
    let cache_client = init_cache().await;
    // for write-back handling
    tokio::spawn(write_behind(Arc::clone(&cache_client), db.clone()));

    let posts_routes = post_routes();
    let posts_cache_routes = post_routes_cache()
        .layer(middleware::from_fn_with_state(
            Arc::clone(&cache_client),
            middleware_cache,
        ));

    let app = Router::new()
        .merge(posts_routes)
        .merge(posts_cache_routes)
        .layer(from_fn(middleware_logger))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}