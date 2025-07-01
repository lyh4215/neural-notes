use axum::{
    Router,
    middleware::{self, from_fn, Next},
    http::{Request, StatusCode, Response},
    extract::{State, Json},
};

use sqlx::SqlitePool;
use tower_http::cors::{CorsLayer};

//auth
use jwt_authorizer::{
    error::InitError, AuthError, Authorizer, IntoLayer, JwtAuthorizer, JwtClaims, Refresh, RefreshStrategy,
};
use serde::{Deserialize, Serialize};
use axum::{
    routing::{get, post, delete, put},
};
mod auth;
use auth::login;
use dotenv::dotenv;


mod db;
use db::init_db;

mod models;
mod routes;

mod posts;

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
    dotenv().ok();

    let db = init_db().await;
    let auth = auth::init_auth().await;
    //let cache_connection = axum_redis_cache::CacheConnection::new(db.clone()).await;
    let cacheconfig = axum_redis_cache::CacheConfig::new()
        .with_url("redis://127.0.0.1")
        .with_write_duration(20);
    
    let cache_connection = axum_redis_cache::CacheConnection::new_with_config(db.clone(), cacheconfig).await;

    let key = String::from("posts");
    let cache_manager = cache_connection.get_manager(key, posts::callback, posts::delete_callback, posts::write_to_cache);

    let protected_routes = Router::new()
        .merge(posts::routes(cache_manager))
        .merge(routes::routes())
        // adding the authorizer layer
        .layer(auth.into_layer());

    let public_routes: Router<SqlitePool> = Router::new()
        .route("/login", post(login))
        .merge(routes::public_routes());

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(from_fn(middleware_logger))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}