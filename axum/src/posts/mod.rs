mod cache;
mod graph;
mod handlers;
mod models;
mod utils;

pub use cache::{callback, delete_callback, write_to_cache};
pub use graph::{get_graph_data, get_related_post};
pub use handlers::{
    __update_post_from_cache, create_post, delete_post, get_posts, list_posts, update_post,
};
pub use models::{
    CreatePost, EmbeddingResponse, GraphData, GraphLink, GraphNode, Post, PostGraphData,
    PostResponse, UpdatePost,
};
pub use utils::internal_error;

use axum::{
    Router,
    middleware::{self},
};

use sqlx::{Postgres, pool::Pool};

use axum_redis_cache::CacheState;

pub fn routes(cache_state: CacheState) -> Router<Pool<Postgres>> {
    Router::new()
        .merge(post_routes_auth())
        .merge(post_routes_cache().layer(middleware::from_fn_with_state(
            cache_state,
            axum_redis_cache::middleware,
        )))
}

fn post_routes_auth() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/posts", axum::routing::get(list_posts).post(create_post))
        .route("/posts/search", axum::routing::get(handlers::search_posts))
        .route("/posts/graph", axum::routing::get(get_graph_data))
}

fn post_routes_cache() -> Router<Pool<Postgres>> {
    Router::new().route(
        "/posts/:id",
        axum::routing::get(get_posts)
            .delete(delete_post)
            .put(update_post),
    )
}
