use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use crate::models::users_mod::Users;
use crate::verify_token::verify;
use crate::utils::encrypt_password; 


// Helper to extract and verify current user
async fn extract_and_verify_user(req: &HttpRequest) -> Result<Users, HttpResponse> {
    let verification_result = verify(req.clone()).await;
    match verification_result {
        Ok(user_data) => {
            match Users::try_from(user_data) {
                Ok(user) => Ok(user),
                Err(_) => Err(HttpResponse::InternalServerError().finish()),
            }
        },
        Err(_) => Err(HttpResponse::Unauthorized().finish())
    }
}

// GET
pub async fn get_user(
    id: web::Path<String>,
    users_collection: web::Data<mongodb::Collection<Users>>,
) -> impl Responder {
    let user_id = match ObjectId::parse_str(&id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let find_result = users_collection.find_one(doc! { "_id": user_id }).await;

    match find_result {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        _ => HttpResponse::InternalServerError().finish(),
    }
}


// GET ALL
pub async fn get_all_users(
    req: HttpRequest,
    users_collection: web::Data<mongodb::Collection<Users>>,
) -> impl Responder {
    match extract_and_verify_user(&req).await {
        Ok(current_user) => {
            if current_user.is_admin.unwrap_or(false) {
                let find_result = users_collection.find(doc! {}).await;

                match find_result {
                    Ok(cursor) => {
                        let users: Vec<Users> = cursor.map(|doc| doc.unwrap()).collect().await;
                        HttpResponse::Ok().json(users)
                    },
                    _ => HttpResponse::InternalServerError().finish(),
                }
            } else {
                HttpResponse::Forbidden().json("You are not allowed to see all users")
            }
        },
        Err(response) => response
    }
}