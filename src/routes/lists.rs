use crate::models::list::List;
use crate::routes::auth::{require_admin, require_auth};
use actix_web::{web, HttpRequest, HttpResponse};
use bson::doc;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::Deserialize;

// ── Query param extractor ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListQuery {
    /// ?type=movie  or  ?type=series
    #[serde(rename = "type")]
    media_type: Option<String>,

    /// ?genre=action
    genre: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /lists  — admin only
pub async fn create_list(
    req: HttpRequest,
    list_data: web::Json<List>,
    list_collection: web::Data<Collection<List>>,
) -> HttpResponse {
    if let Err(res) = require_admin(req).await {
        return res;
    }

    match list_collection.insert_one(list_data.into_inner()).await {
        Ok(result) => HttpResponse::Created().json(result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// DELETE /lists/{id}  — admin only
pub async fn delete_list(
    req: HttpRequest,
    list_id: web::Path<String>,
    list_collection: web::Data<Collection<List>>,
) -> HttpResponse {
    if let Err(res) = require_admin(req).await {
        return res;
    }

    match list_collection
        .delete_one(doc! { "_id": list_id.into_inner() })
        .await
    {
        Ok(result) if result.deleted_count == 0 => HttpResponse::NotFound().body("List not found"),
        Ok(_) => HttpResponse::Ok().body("The list has been deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /lists?type=movie&genre=action  — any authenticated user
pub async fn get_lists(
    req: HttpRequest,
    query: web::Query<ListQuery>,
    list_collection: web::Data<Collection<List>>,
) -> HttpResponse {
    if let Err(res) = require_auth(req).await {
        return res;
    }

    let filter = match (&query.media_type, &query.genre) {
        (Some(t), Some(g)) => doc! { "type": t, "genre": g },
        (Some(t), None) => doc! { "type": t },
        (None, Some(g)) => doc! { "genre": g },
        (None, None) => doc! {},
    };

    match list_collection.find(filter).await {
        Ok(cursor) => match cursor.try_collect::<Vec<List>>().await {
            Ok(lists) => HttpResponse::Ok().json(lists),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}