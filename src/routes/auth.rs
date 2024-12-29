#![allow(deprecated)]

use actix_web::{web, HttpResponse, Responder};
use base64::engine::general_purpose::STANDARD;
use base64::{decode, Engine};
use bson::doc;
use jsonwebtoken::{encode, EncodingKey, Header};
use models::user_mod::User;
use mongodb::Collection;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::env;

use crate::models;
use crate::utils::{decrypt_password, encrypt_password, get_secret_key};

// Claims struct to store the token claims 
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub is_admin: bool,
}

// Register a new user 
pub async fn register_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<User>,
) -> impl Responder {

    // Encrypt the password 
    let encrypted_password = match encrypt_password(&user_info.password) {
        Ok(password) => password,
        Err(error_message) => {
            eprintln!("{}", error_message);
            return HttpResponse::InternalServerError().body(error_message);
        }
    };

    // Create a new user with the encrypted password 
    let new_user = User {
        username: user_info.username.clone(),
        email: user_info.email.clone(),
        password: encrypted_password,
        profile_pic: user_info.profile_pic.clone(),
        is_admin: Some(true),
    };

    // Insert the new user into the database 
    match auth_db.insert_one(new_user).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


// Login a user 
pub async fn login_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<User>,
) -> impl Responder {

    // Check if the password is empty 
    if user_info.password.is_empty() {
        return HttpResponse::BadRequest().body("Password is required.");
    }

    // Create a query to find the user in the database 
    let query = user_info.email.as_ref().map_or_else(
        || user_info.username.as_ref().map_or_else(
            || doc! {},
            |username| doc! { "username": username }
        ),
        |email| doc! { "email": email }
    );

    // Check if the query is empty 
    if query.is_empty() {
        return HttpResponse::BadRequest().body("Email or username is required.");
    }

    // Find the user in the database  
    let user_result = auth_db.find_one(query).await;
    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().body("Wrong password or username!"),
        Err(_) => return HttpResponse::InternalServerError().body("Database query failed"),
    };

    // Get the secret key
    let secret_key = match get_secret_key() {
        Ok(key) => key,
        Err(response) => return response,
    };

    // Decrypt the password 
    let password_decrypted = match decrypt_password(&user.password, &secret_key) {
        Ok(decrypted) => decrypted,
        Err(response) => return response,
    };

    // Check if the decrypted password is the same as the user's password 
    if password_decrypted != user_info.password {
        return HttpResponse::Unauthorized().body("Wrong password or username!");
    }

    // Create the claims for the token 
    let claims = Claims {
        sub: user.email.unwrap_or_default(),
        exp: 1000000000, 
        is_admin: user.is_admin.unwrap_or(false),
    };

    // Generate the token 
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret_key)) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to generate token"),
    };

    // Return the token 
    HttpResponse::Ok().json(token)
}

