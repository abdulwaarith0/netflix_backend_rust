use crate::models::user::User;
use crate::utils::{decrypt_password, encrypt_password, get_secret_key};
use crate::verify_token::verify;
use actix_web::{web, HttpRequest, HttpResponse};
use bson::doc;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::Collection;
use serde::{Deserialize, Serialize};

// ── Token claims ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user's email
    pub exp: usize,
    pub is_admin: bool,
}

// ── Shared auth guards ────────────────────────────────────────────────────────

/// Verifies the request token and confirms the caller is an admin.
/// Returns `Ok(())` on success, or the ready-to-return `HttpResponse` on failure.
pub async fn require_admin(req: HttpRequest) -> Result<(), HttpResponse> {
    match verify(req).await {
        Ok(claims) => {
            if claims.get("is_admin").map(|v| v == "true").unwrap_or(false) {
                Ok(())
            } else {
                Err(HttpResponse::Forbidden().body("You are not allowed!"))
            }
        }
        Err(_) => Err(HttpResponse::Unauthorized().finish()),
    }
}

/// Verifies the request token only — no admin check.
pub async fn require_auth(req: HttpRequest) -> Result<(), HttpResponse> {
    match verify(req).await {
        Ok(_) => Ok(()),
        Err(_) => Err(HttpResponse::Unauthorized().finish()),
    }
}

// ── Register ──────────────────────────────────────────────────────────────────

/// POST /auth/register
pub async fn register_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<User>,
) -> HttpResponse {
    if user_info.email.is_empty() {
        return HttpResponse::BadRequest().body("Email is required.");
    }

    if user_info.password.is_empty() {
        return HttpResponse::BadRequest().body("Password is required.");
    }

    let encrypted_password = match encrypt_password(&user_info.password) {
        Ok(pw) => pw,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let new_user = User {
        id: None,
        username: user_info.username.clone(),
        email: user_info.email.clone(),
        password: encrypted_password,
        profile_pic: user_info.profile_pic.clone(),
        is_admin: false,
    };

    match auth_db.insert_one(new_user).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// ── Login ─────────────────────────────────────────────────────────────────────

/// Dedicated input struct for login — accept either email or username,
#[derive(Deserialize)]
pub struct LoginInput {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

/// POST /auth/login
pub async fn login_user(
    auth_db: web::Data<Collection<User>>,
    user_info: web::Json<LoginInput>,
) -> HttpResponse {
    if user_info.password.is_empty() {
        return HttpResponse::BadRequest().body("Password is required.");
    }

    let query = match (&user_info.email, &user_info.username) {
        (Some(email), _) if !email.is_empty() => doc! { "email": email },
        (_, Some(username)) if !username.is_empty() => doc! { "username": username },
        _ => return HttpResponse::BadRequest().body("Email or username is required."),
    };

    let user = match auth_db.find_one(query).await {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::Unauthorized().body("Wrong credentials."),
        Err(_) => return HttpResponse::InternalServerError().body("Database query failed."),
    };

    let secret_key = match get_secret_key() {
        Ok(k) => k,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let decrypted = match decrypt_password(&user.password, &secret_key) {
        Ok(pw) => pw,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if decrypted != user_info.password {
        return HttpResponse::Unauthorized().body("Wrong credentials.");
    }

    let claims = Claims {
        sub: user.email,
        exp: 1_000_000_000,
        is_admin: user.is_admin,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret_key),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to generate token."),
    };

    HttpResponse::Ok().json(token)
}