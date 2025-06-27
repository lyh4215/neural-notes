use axum::{
    Router,
    middleware::{self, from_fn, Next},
    http::{Request, StatusCode, Response},
    extract::{State, Json},
};

use futures_util::task::ArcWake;
use sqlx::SqlitePool;
use tower_http::cors::{CorsLayer};

//cache
mod cache;
use cache::{init_cache, write_behind, delete_event_listener, middleware_cache};

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


use std::sync::Arc;
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
    // build our application with a single route
    let db = init_db().await;
    let client = init_cache().await;

    //redis-rs의 conn은 clone시 내부적으로 connection을 하나로 유지하기 때문에, clone해서 넘겨주어도 상관없음.
    //handle(conn)은 쓰레드간 공유 필요 x
    let conn = client.get_multiplexed_async_connection().await.unwrap();
    // for write-back handling
    tokio::spawn(write_behind(conn.clone(), db.clone()));
    tokio::spawn(delete_event_listener(client.clone(), db.clone()));
    
    let auth : Authorizer<auth::UserClaims> = JwtAuthorizer::from_secret(&std::env::var("JWT_SECRET").expect("JWT_SECRET not set"))
        .build()
        .await.unwrap();

    let protected_routes = Router::new()
        .merge(posts::routes(conn.clone()))
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