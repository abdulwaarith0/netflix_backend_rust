#![allow(dead_code)]

use actix_web::HttpResponse;
use actix_web::{web::Data, HttpRequest, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::collections::HashMap;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::env;
use thiserror::Error;

use crate::auth::Claims;

// Get the token from the cookie 
pub fn get_token_from_cookie(req: &HttpRequest) -> Option<String> {
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

// Get the token from the authorization header
pub fn get_token_from_auth_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(actix_web::http::header::PROXY_AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.split(' ').nth(1).map(String::from))
}

// Get the token from the cookie or the authorization header
pub fn get_jwt_token(req: &HttpRequest) -> Option<String> {
    get_token_from_cookie(req).or_else(|| get_token_from_auth_header(req))
}

// Verify the token
pub async fn verify(req: HttpRequest) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let token = get_jwt_token(&req).ok_or(AppError::TokenNotFound)?;
    let secret_key = env::var("SECRET_KEY").map_err(|_| AppError::SecretKeyNotSet)?;
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    ).map_err(|e| e.to_string())?;

    // Convert the claims to a map of strings
    let mut claims_map = HashMap::new();
    claims_map.insert("sub".to_string(), token_data.claims.sub);
    claims_map.insert("exp".to_string(), token_data.claims.exp.to_string());
    claims_map.insert("is_admin".to_string(), token_data.claims.is_admin.to_string());

    Ok(claims_map)
}

// Error handling for the token verification
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