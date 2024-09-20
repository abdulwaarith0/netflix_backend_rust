use actix_web::{web, HttpResponse, HttpRequest, Responder};
use bson::doc;
use mongodb::Collection;
use futures_util::{stream::StreamExt, TryStreamExt as _};
use crate::models::list_mod::List;
use crate::verify_token::verify;
use serde_json::json;

// CREATE LIST
pub async fn create_list(
    req: HttpRequest,
    list_data: web::Json<List>,
    list_collection: web::Data<Collection<List>>,
) -> impl Responder {
    let claims = verify(req).await;
    if claims.is_ok() && claims.unwrap().get("is_admin") == Some(&"true".to_string()) {
        match list_collection.insert_one(list_data.into_inner()).await {
            Ok(result) => HttpResponse::Created().json(result.inserted_id),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::Forbidden().json("You are not allowed!")
    }
}

// DELETE LIST
pub async fn delete_list(
    req: HttpRequest,
    list_id: web::Path<String>,
    list_collection: web::Data<Collection<List>>,
) -> impl Responder {
    let claims = verify(req).await;
    if claims.is_ok() && claims.unwrap().get("is_admin") == Some(&"true".to_string()) {
        match list_collection.delete_one(doc! { "_id": list_id.into_inner() }).await {
            Ok(_) => HttpResponse::Ok().json("The list has been deleted..."),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::Forbidden().json("You are not allowed!")
    }
}

// GET LISTS
pub async fn get_lists(
    req: HttpRequest,
    list_collection: web::Data<Collection<List>>,
) -> impl Responder {
    let claims = verify(req.clone()).await;
    if claims.is_err() {
        return HttpResponse::Unauthorized().finish();
    }

    let type_query = req.query_string().contains("type=");
    let genre_query = req.query_string().contains("genre=");

    let filter = if type_query && genre_query {
        doc! { "type": req.query_string(), "genre": req.query_string() }
    } else if type_query {
        doc! { "type": req.query_string() }
    } else {
        doc! {}
    };

    let cursor = list_collection.find(filter).await;
    match cursor {
        Ok(mut cursor) => {
            let mut lists = Vec::new();
            while let Some(list) = cursor.try_next().await.unwrap_or(None) {
                lists.push(list);
            }
            HttpResponse::Ok().json(lists)
        },
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}