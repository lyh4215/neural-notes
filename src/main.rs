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
use cache::{init_cache, write_behind, middleware_cache};

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
use routes::{post_routes, post_routes_cache, post_routes_auth};

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
async fn middleware_auth(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let headers = req.headers();
    let auth_value = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    match auth_value {
        Some(val) => {
            println!("헤더 존재! {val}");
        }
        None => println!("헤더 없음"),
    }

    let response = next.run(req).await;
    Ok(response)
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    // build our application with a single route
    let db = init_db().await;
    let cache_client = init_cache().await;
    // for write-back handling
    tokio::spawn(write_behind(Arc::clone(&cache_client), db.clone()));
    

    // First let's create an authorizer builder from a Oidc Discovery
    // User is a struct deserializable from JWT claims representing the authorized user
    // let jwt_auth: JwtAuthorizer<User> = JwtAuthorizer::from_oidc("https://accounts.google.com/")
    let auth : Authorizer<auth::UserClaims> = JwtAuthorizer::from_secret(&std::env::var("JWT_SECRET").expect("JWT_SECRET not set"))
        .build()
        .await.unwrap();

    let posts_routes = post_routes();
    let posts_cache_routes = post_routes_cache()
        .layer(middleware::from_fn_with_state(
            Arc::clone(&cache_client),
            middleware_cache,
        ));
    let posts_routes_auth = post_routes_auth();

    let authorized_routes = Router::new()
        .merge(posts_routes)
        .merge(posts_cache_routes)
        .merge(posts_routes_auth)
        .route("/protected", get(protected))
        // adding the authorizer layer
        .layer(auth.into_layer());

    let unauthorized_routes: Router<SqlitePool> = Router::new()
        .route("/login", post(login));

    let app = Router::new()
        .merge(authorized_routes)
        .merge(unauthorized_routes)
        .layer(from_fn(middleware_logger))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


/// handler with injected claims object
async fn protected(JwtClaims(user): JwtClaims<auth::UserClaims>) -> Result<String, AuthError> {
    // Send the protected data to the user
    Ok(format!("Welcome: {} {}", user.sub, user.username))
}

