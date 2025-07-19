use axum::{
    Router,
    extract::{Json, State},
    http::{Request, Response, StatusCode},
    middleware::{self, Next, from_fn},
};

use sqlx::{Postgres, pool::Pool};
use tower_http::cors::CorsLayer;

use tokio_util::sync::CancellationToken;

//auth
use axum::routing::{delete, get, post, put};
use jwt_authorizer::{
    AuthError, Authorizer, IntoLayer, JwtAuthorizer, JwtClaims, Refresh, RefreshStrategy,
    error::InitError,
};
use serde::{Deserialize, Serialize};
mod auth;
use auth::login;
use dotenv::dotenv;

mod db;
use db::init_db;

mod models;
mod routes;

mod posts;

use axum::body::Body;

async fn middleware_logger(req: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode> {
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
    let cacheconnconfig = axum_redis_cache::CacheConnConfig::new().with_url(
        std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set")
            .as_str(),
    );

    let cache_connection =
        axum_redis_cache::CacheConnection::new_with_config(db.clone(), cacheconnconfig).await;

    let key = String::from("posts");
    let cache_manager = cache_connection.get_manager(
        key,
        posts::callback,
        posts::delete_callback,
        posts::write_to_cache,
    );

    let protected_routes = Router::new()
        .merge(posts::routes(cache_manager.get_state()))
        .merge(routes::routes())
        // adding the authorizer layer
        .layer(auth.into_layer());

    let public_routes: Router<Pool<Postgres>> = Router::new()
        .route("/login", post(login))
        .merge(routes::public_routes());

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(from_fn(middleware_logger))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c");
            println!("🔴 Ctrl+C received. Cancelling...");
        })
        .await
        .unwrap();

    cache_manager.shutdown().await;
}
