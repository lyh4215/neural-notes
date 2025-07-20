// src/posts.rs

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    middleware::{self, Next, from_fn},
    routing::{delete, get, post, put},
};

use jwt_authorizer::JwtClaims;

use crate::{
    auth::UserClaims,
    models::{CreatePost, Post, PostGraphData, PostResponse, UpdatePost, User},
};
use redis::{Client, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, pool::Pool};

//cache
use axum_redis_cache::{CacheManager, CacheState};

//embedding
use pgvector::Vector;

//graph
use crate::models::{GraphData, GraphLink, GraphNode};

use std::time::Duration;
use tokio::time::sleep;

pub fn routes(
    //mut conn: MultiplexedConnection
    cache_state: CacheState,
) -> Router<Pool<Postgres>> {
    Router::new()
        .merge(post_routes_auth())
        .merge(post_routes_cache().layer(middleware::from_fn_with_state(
            cache_state,
            axum_redis_cache::middleware,
        )))
}

fn post_routes_auth() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/graph", get(get_graph_data))
}

fn post_routes_cache() -> Router<Pool<Postgres>> {
    Router::new().route(
        "/posts/:id",
        get(get_posts).delete(delete_post).put(update_post),
    )
}

pub fn write_to_cache(old: String, new: String) -> String {
    let parsed_body: UpdatePost = serde_json::from_str(&new).unwrap();
    let mut payload: PostResponse = serde_json::from_str(&old).unwrap();

    if let Some(content) = parsed_body.content {
        payload.content = content;
    }
    if let Some(title) = parsed_body.title {
        payload.title = title;
    }
    serde_json::to_string(&payload).unwrap()
}

pub async fn callback(db: Pool<Postgres>, value: String) {
    use crate::models::PostResponse;
    let json: PostResponse = serde_json::from_str(&value).unwrap();
    use crate::models::UpdatePost;
    use crate::posts::__update_post_from_cache;
    //TODO : from PostResponse, to UpdatePost
    let update_json = UpdatePost {
        title: Some(json.title),
        content: Some(json.content),
    };
    let _ = __update_post_from_cache(State(db), Path(json.id), Json(update_json)).await;
}

pub async fn delete_callback(db: Pool<Postgres>, key: String) {
    if let Ok(post_id) = key.parse::<i64>() {
        println!("🧹 expired 감지됨: delete marker for post_id={}", post_id);

        // 실제 DB에서 삭제
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(post_id)
            .execute(&db)
            .await
            .unwrap();

        println!(
            "✅ DB에서 post {} 삭제 완료 ({} rows affected)",
            post_id,
            result.rows_affected()
        );
    }
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

async fn create_post(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;
    let post = sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (title, content, user_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&user.sub)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Error inserting post: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create post".to_string(),
        )
    })?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(post))
}

async fn list_posts(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Json<Vec<Post>> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_id = $1 
    ORDER BY created_at DESC;",
    )
    .bind(user.sub)
    .fetch_all(&db)
    .await
    .unwrap();

    //TODO : unwarp 사용 안하는게 좋음
    Json(posts)
}
async fn get_posts(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.sub)
        .fetch_one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let related_posts = get_related_post(&post, &db).await;
    let post_response = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at,
        updated_at: post.updated_at,
        user_id: post.user_id,
        related_posts,
    };

    Ok(Json(post_response))
}
async fn delete_post(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query("DELETE FROM posts WHERE id = $1 and user_id = $2")
        .bind(id)
        .bind(user.sub)
        .execute(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    Ok(())
}

//todo : make this
pub async fn update_post(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let mut tx = db.begin().await.map_err(internal_error)?;

    sqlx::query("UPDATE posts SET title = $1, content = $2 WHERE id = $3 AND user_id = $4")
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(id)
        .bind(user.sub)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    tx.commit().await.map_err(internal_error)?;

    let related_posts = get_related_post(&post, &db).await;

    let post_response: PostResponse = PostResponse {
        id: post.id,
        title: post.title,
        content: post.content,
        created_at: post.created_at,
        updated_at: post.updated_at,
        user_id: post.user_id,
        related_posts,
    };
    Ok(Json(post_response))
}

pub async fn __update_post_from_cache(
    State(db): State<Pool<Postgres>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePost>,
) {
    let mut tx = db.begin().await.map_err(internal_error).unwrap();
    let client = reqwest::Client::new();
    let embed_api_url =
        std::env::var("EMBED_API_URL").unwrap_or_else(|_| "http://embed_api:8001".to_string());
    let max_retries = 3;
    let retry_delay = Duration::from_secs(10); // ⏱️ 재시도 간격: 10초
    let mut last_err = None;

    let embed_resp = {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match client
                .post(format!("{}/embed", embed_api_url))
                .json(&serde_json::json!({ "text": payload.content }))
                .send()
                .await
            {
                Ok(resp) => break Ok(resp),
                Err(e) => {
                    last_err = Some(e.to_string());
                    if attempt >= max_retries {
                        break Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!(
                                "Failed after {} attempts: {}, URL: {}",
                                attempt,
                                last_err.clone().unwrap_or_default(),
                                embed_api_url
                            ),
                        ));
                    }
                    eprintln!(
                        "⚠️ Request failed (attempt {}): {}. Retrying in {}s...",
                        attempt,
                        e,
                        retry_delay.as_secs()
                    );
                    sleep(retry_delay).await;
                }
            }
        }
    }
    .unwrap();

    let embed_resp: EmbedResponse = embed_resp
        .json()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .unwrap();

    // ➋ pgvector Vector로 변환
    let vector = Vector::from(embed_resp.embedding);

    sqlx::query(
        r#"
        UPDATE posts 
        SET title = $1, content = $2 , embedding = $3
        WHERE id = $4
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(vector)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Post not found".to_string()))
    .unwrap();

    tx.commit().await.map_err(internal_error).unwrap();
}

async fn get_related_post(post: &Post, db: &Pool<Postgres>) -> Vec<Post> {
    /* related post 가져오기기 */
    let Some(embedding) = &post.embedding else {
        return Vec::new();
    };
    // 유사도 기반으로 관련 포스트 3개 반환
    let related_posts = sqlx::query_as::<_, Post>(
        r#"
        SELECT * FROM posts
        WHERE user_id = $1
        AND id != $2
        AND embedding IS NOT NULL
        ORDER BY embedding <-> $3
        LIMIT 3
        "#,
    )
    .bind(post.user_id)
    .bind(post.id)
    .bind(embedding)
    .fetch_all(db)
    .await
    .unwrap_or_else(|e| {
        eprintln!("DB error during related post fetch: {}", e);
        Vec::new()
    });

    related_posts
}

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("에러: {}", e))
}

async fn get_graph_data(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<Json<GraphData>, (StatusCode, String)> {
    let posts = sqlx::query_as::<_, PostGraphData>(
        "SELECT id, title, embedding FROM posts WHERE user_id = $1 AND embedding IS NOT NULL",
    )
    .bind(user.sub)
    .fetch_all(&db)
    .await
    .map_err(internal_error)?;

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut links: Vec<GraphLink> = Vec::new();

    for post in &posts {
        nodes.push(GraphNode {
            id: post.id.to_string(),
            name: post.title.clone(),
        });
    }

    // Calculate similarities and create links
    for i in 0..posts.len() {
        for j in (i + 1)..posts.len() {
            let p1 = &posts[i];
            let p2 = &posts[j];

            if let (Some(e1), Some(e2)) = (&p1.embedding, &p2.embedding) {
                // Calculate cosine distance (pgvector uses <-> for cosine distance)
                //println!("Calculating distance between post {} and {}", p1.id, p2.id);
                let distance = sqlx::query_scalar::<_, f64>("SELECT $1 <-> $2")
                    .bind(e1)
                    .bind(e2)
                    .fetch_one(&db)
                    .await
                    .map_err(|e| {
                        eprintln!("Error calculating distance: {}", e);
                        internal_error(e)
                    })?;
                //println!("Distance: {}", distance);

                // Only add link if similarity is above a certain threshold
                // Cosine distance ranges from 0 to 2. 0 means identical, 2 means opposite.
                // A distance of < 0.2 means similarity > 0.8 (since similarity = 1 - distance/2)
                if distance < 0.5 {
                    links.push(GraphLink {
                        source: p1.id.to_string(),
                        target: p2.id.to_string(),
                        value: (1.0 - (distance / 2.0)) as f32, // Convert distance to similarity (0 to 1)
                    });
                }
            }
        }
    }

    Ok(Json(GraphData { nodes, links }))
}
