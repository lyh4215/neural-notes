
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
        extract::{State,Json},
        http::{Request, StatusCode, Response},
    };
use serde::{Serialize, Deserialize};
use bcrypt::verify;
use sqlx::SqlitePool;

use crate::models::User;

#[derive(Serialize)]
struct TokenClaims {
    sub: i64, //id
    username: String,
    exp: usize,
}

use jwt_authorizer::{
    error::InitError, AuthError, Authorizer, IntoLayer, JwtAuthorizer, JwtClaims, Refresh, RefreshStrategy,
};

//for auth
/// Object representing claims
/// (a subset of deserialized claims)
#[derive(Debug, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: i64,
    pub username: String,
}

#[derive(serde::Deserialize)]
pub struct UserLogin {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
pub struct AccessToken {
    access_token : String,
}
pub async fn init_auth() -> Authorizer<UserClaims>{
    JwtAuthorizer::from_secret(&std::env::var("JWT_SECRET").expect("JWT_SECRET not set"))
    .build()
    .await.unwrap()
}
pub async fn login(
    State(db): State<SqlitePool>,
    Json(payload) : Json<UserLogin>)
-> Result<Json<AccessToken>, (StatusCode, String)> {
    //db에서 찾기
    let user : User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(payload.username)
        .fetch_one(&db)
        .await
        .unwrap();

    // 비밀번호 검증 (DB hash vs 입력값)
    let valid = verify(&payload.password, &user.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "hash error".to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "invalid password".to_string()));
    }

    //jwt token 발급
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let exp = now + 6000; //1h
    let claims = TokenClaims {
        sub: user.id,
        username: user.username.to_string(),
        exp: exp as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();
    Ok(Json(AccessToken { access_token: token }))
}
