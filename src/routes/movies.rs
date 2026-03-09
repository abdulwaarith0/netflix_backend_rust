use crate::routes::auth::{require_admin, require_auth};
use crate::models::movie::Movie;
use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt;
use mongodb::{bson::doc, Collection};
use serde::Deserialize;

// ── Query param extractor ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct MovieTypeQuery {
    /// ?type=series  or  ?type=movie
    #[serde(rename = "type")]
    media_type: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /movies  — admin only
pub async fn create_movie(
    req: HttpRequest,
    movie_data: web::Json<Movie>,
    movie_collection: web::Data<Collection<Movie>>,
) -> HttpResponse {
    if let Err(res) = require_admin(req).await {
        return res;
    }

    match movie_collection.insert_one(movie_data.into_inner()).await {
        Ok(result) => HttpResponse::Created().json(result.inserted_id),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /movies  — admin only
pub async fn get_all_movies(
    req: HttpRequest,
    movie_collection: web::Data<Collection<Movie>>,
) -> HttpResponse {
    if let Err(res) = require_admin(req).await {
        return res;
    }

    match movie_collection.find(doc! {}).await {
        Ok(cursor) => match cursor.try_collect::<Vec<Movie>>().await {
            Ok(movies) => HttpResponse::Ok().json(movies),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /movies/{id}  — any authenticated user
pub async fn get_movie(
    req: HttpRequest,
    movie_id: web::Path<String>,
    movie_collection: web::Data<Collection<Movie>>,
) -> HttpResponse {
    if let Err(res) = require_auth(req).await {
        return res;
    }

    let filter = doc! { "_id": movie_id.into_inner() };

    match movie_collection.find_one(filter).await {
        Ok(Some(movie)) => HttpResponse::Ok().json(movie),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// GET /movies/random?type=series  — any authenticated user
pub async fn get_random_movie(
    req: HttpRequest,
    query: web::Query<MovieTypeQuery>,
    movie_collection: web::Data<Collection<Movie>>,
) -> HttpResponse {
    if let Err(res) = require_auth(req).await {
        return res;
    }

    let is_series = query.media_type.as_deref() == Some("series");

    let pipeline = vec![
        doc! { "$match": { "is_series": is_series } },
        doc! { "$sample": { "size": 1 } },
    ];

    match movie_collection.aggregate(pipeline).await {
        Ok(mut cursor) => match cursor.try_next().await {
            Ok(Some(doc)) => HttpResponse::Ok().json(doc),
            Ok(None) => HttpResponse::NotFound().finish(),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}