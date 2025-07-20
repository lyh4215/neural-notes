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
