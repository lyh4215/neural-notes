use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use jwt_authorizer::JwtClaims;
use pgvector::Vector;
use sqlx::{Postgres, pool::Pool};

use super::models::{GraphData, GraphLink, GraphNode, Post, PostGraphData};
use super::utils::internal_error;
use crate::auth::UserClaims;

// Minimum similarity threshold for related posts (0.0 to 1.0)
const MIN_SIMILARITY_THRESHOLD: f64 = 0.7;
// Corresponding maximum distance (0.0 to 2.0)
const MAX_DISTANCE_THRESHOLD: f64 = 2.0 * (1.0 - MIN_SIMILARITY_THRESHOLD);

pub async fn get_related_post(post: &Post, db: &Pool<Postgres>) -> Vec<Post> {
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

pub async fn get_graph_data(
    State(db): State<Pool<Postgres>>,
    JwtClaims(user): JwtClaims<UserClaims>,
) -> Result<Json<GraphData>, (StatusCode, String)> {
    let posts = sqlx::query_as::<_, PostGraphData>(
        r#"SELECT id, title, embedding
            FROM posts
            WHERE user_id = $1
            AND embedding IS NOT NULL"#,
    )
    .bind(user.sub)
    .fetch_all(&db)
    .await
    .map_err(internal_error)?;

    let nodes: Vec<GraphNode> = posts
        .iter()
        .map(|post| GraphNode {
            id: post.id.to_string(),
            name: post.title.clone(),
        })
        .collect();

    // Helper struct for query result
    #[derive(sqlx::FromRow)]
    struct SimilarPair {
        source: i64,
        target: i64,
        distance: Option<f64>,
    }

    // Calculate similarities and create links in a single query
    let similar_pairs = sqlx::query_file_as!(
        SimilarPair,
        "sql/get_related_posts.sql",
        user.sub,
        MAX_DISTANCE_THRESHOLD
    )
    .fetch_all(&db)
    .await
    .map_err(internal_error)?;

    let links: Vec<GraphLink> = similar_pairs
        .into_iter()
        .filter_map(|pair| {
            pair.distance.map(|dist| GraphLink {
                source: pair.source.to_string(),
                target: pair.target.to_string(),
                // 코사인 distance(0~2) → similarity(0~1)
                value: (1.0 - (dist / 2.0)) as f32,
            })
        })
        .collect();

    Ok(Json(GraphData { nodes, links }))
}
