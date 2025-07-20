use axum::extract::{Json, Path, State};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, pool::Pool};

use super::handlers::__update_post_from_cache;
use super::models::{PostResponse, UpdatePost};

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
    use crate::posts::{PostResponse, UpdatePost};
    let json: PostResponse = serde_json::from_str(&value).unwrap();
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
