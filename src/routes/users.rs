use crate::routes::auth::{require_admin, require_auth};
use crate::models::users::Users;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /users/{id}  — any authenticated user
pub async fn get_user(
    req: HttpRequest,
    id: web::Path<String>,
    users_collection: web::Data<mongodb::Collection<Users>>,
) -> HttpResponse {
    if let Err(res) = require_auth(req).await {
        return res;
    }

    let user_id = match ObjectId::parse_str(id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format."),
    };

    match users_collection.find_one(doc! { "_id": user_id }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found."), // was InternalServerError
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /users  — admin only
pub async fn get_all_users(
    req: HttpRequest,
    users_collection: web::Data<mongodb::Collection<Users>>,
) -> HttpResponse {
    if let Err(res) = require_admin(req).await {
        return res;
    }

    match users_collection.find(doc! {}).await {
        Ok(cursor) => match cursor.try_collect::<Vec<Users>>().await {
            Ok(users) => HttpResponse::Ok().json(users),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}