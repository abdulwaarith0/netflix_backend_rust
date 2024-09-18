use actix_web::{web::Data, HttpRequest, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use thiserror::Error;

use crate::auth::Claims;

fn get_token_from_cookie(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(actix_web::http::header::COOKIE)
        .and_then(|header| header.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let mut parts = cookie.trim().split('=');
                if parts.next()? == "token" {
                    parts.next().map(String::from)
                } else {
                    None
                }
            })
        })
}

fn get_token_from_auth_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(actix_web::http::header::PROXY_AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.split(' ').nth(1).map(String::from))
}

fn get_jwt_token(req: &HttpRequest) -> Option<String> {
    get_token_from_cookie(req).or_else(|| get_token_from_auth_header(req))
}

async fn verify(req: HttpRequest) -> Result<(), Box<dyn std::error::Error>> {
    let token = get_jwt_token(&req).ok_or(AppError::TokenNotFound)?;
    let secret_key = env::var("SECRET_KEY").map_err(|_| AppError::SecretKeyNotSet)?;
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    ).map_err(|e| AppError::DecodeError(e.to_string()))?;

    req.extensions_mut().insert(token_data.claims);
    Ok(())
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Token not found")]
    TokenNotFound,
    #[error("Secret key not set")]
    SecretKeyNotSet,
    #[error("Failed to decode token: {0}")]
    DecodeError(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}