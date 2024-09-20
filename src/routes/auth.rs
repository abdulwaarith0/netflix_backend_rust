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

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub is_admin: bool,
}

pub async fn register_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<User>,
) -> impl Responder {
    let cipher = Cipher::aes_256_cbc();

    let secret_key = match env::var("SECRET_KEY") {
        Ok(encoded_key) => match decode(&encoded_key) {
            Ok(key) => {
                if key.len() == 32 {
                    key
                } else {
                    eprintln!("Secret key must be exactly 32 bytes after decoding.");
                    return HttpResponse::InternalServerError().body("Invalid SECRET_KEY length");
                }
            }
            Err(e) => {
                eprintln!("Failed to decode SECRET_KEY: {:?}", e);
                return HttpResponse::InternalServerError().body("Failed to decode SECRET_KEY");
            }
        },
        Err(_) => return HttpResponse::InternalServerError().body("SECRET_KEY not set"),
    };

    // Safely perform encryption
    let password_encrypted =
        match encrypt(cipher, &secret_key, None, &user_info.password.as_bytes()) {
            Ok(encrypted) => encrypted,
            Err(e) => {
                eprintln!("Encryption error: {:?}", e); 
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to encrypt password: {:?}", e));
            }
        };

    let new_user = User {
        username: user_info.username.clone(),
        email: user_info.email.clone(),
        password: STANDARD.encode(password_encrypted),
        profile_pic: user_info.profile_pic.clone(),
        is_admin: Some(true),
    };

    match auth_db.insert_one(new_user).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn login_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<User>,
) -> impl Responder {
    // Check if the password is provided
    if user_info.password.is_empty() {
        return HttpResponse::BadRequest().body("Password is required.");
    }

    // Determine the query based on provided info
    let query = if let Some(email) = &user_info.email {
        doc! { "email": email }
    } else if let Some(username) = &user_info.username {
        doc! { "username": username }
    } else {
        return HttpResponse::BadRequest().body("Email or username is required.");
    };
    let user_result = auth_db.find_one(query).await;

    match user_result {
        Ok(Some(user)) => {
            let secret_key = match env::var("SECRET_KEY") {
                Ok(encoded_key) => match decode(&encoded_key) {
                    Ok(key) => {
                        if key.len() == 32 {
                            key
                        } else {
                            return HttpResponse::InternalServerError().body("Invalid key length");
                        }
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .body("Failed to decode SECRET_KEY")
                    }
                },
                Err(_) => return HttpResponse::InternalServerError().body("SECRET_KEY not set"),
            };

            let password_decrypted = match decrypt(
                Cipher::aes_256_cbc(),
                &secret_key,
                None,
                &decode(&user.password).unwrap_or_default(),
            ) {
                Ok(decrypted) => decrypted,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("Failed to decrypt password")
                }
            };

            if String::from_utf8(password_decrypted).unwrap_or_default() != user_info.password {
                return HttpResponse::Unauthorized().body("Wrong password or username!!");
            }
            let claims = Claims {
                sub: user.email.clone().unwrap_or_default(),
                exp: 10000000,
                is_admin: user.is_admin.unwrap_or(false),
            };

            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(&secret_key),
            ) {
                Ok(t) => t,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("Failed to generate token")
                }
            };
            HttpResponse::Ok().json(token)
        }
        Ok(None) => HttpResponse::Unauthorized().body("Wrong password or username!!"),
        Err(_) => HttpResponse::InternalServerError().body("Database query failed"),
    }
}
