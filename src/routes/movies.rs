use crate::models::movie_mod::Movie;
use crate::verify_token::verify;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures_util::{stream::StreamExt, TryStreamExt as _};
use mongodb::{
    bson::{doc, Bson},
    Client, Collection,
};
use serde_json::json;

// CREATE MOVIE
pub async fn create_movie(
    req: HttpRequest,
    movie_data: web::Json<Movie>,
    movie_collection: web::Data<Collection<Movie>>,
) -> impl Responder {
    if let Ok(claims) = verify(req).await {
        if let Some(is_admin_str) = claims.get("is_admin") {
            let is_admin = is_admin_str == "true";
            if is_admin {
                match movie_collection.insert_one(movie_data.into_inner()).await {
                    Ok(result) => HttpResponse::Created().json(result.inserted_id),
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            } else {
                HttpResponse::Forbidden().body("You are not allowed!")
            }
        } else {
            HttpResponse::Forbidden().body("You are not allowed!")
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

// GET ALL MOVIES
pub async fn get_all_movies(
    req: HttpRequest,
    movie_collection: web::Data<Collection<Movie>>,
) -> impl Responder {
    if let Ok(claims) = verify(req).await {
        if let Some(is_admin_str) = claims.get("is_admin") {
            let is_admin = is_admin_str == "true";
            if is_admin {
                let cursor = movie_collection.find(doc! {}).await;
                match cursor {
                    Ok(mut cursor) => {
                        let mut movies = Vec::new();
                        while let Some(movie) = cursor.try_next().await.unwrap_or(None) {
                            movies.push(movie);
                        }
                        HttpResponse::Ok().json(movies)
                    }
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
                }
            } else {
                HttpResponse::Forbidden().body("You are not allowed!")
            }
        } else {
            HttpResponse::Forbidden().body("You are not authorized!")
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}


// GET MOVIE
pub async fn get_movie(
    req: HttpRequest,
    movie_id: web::Path<String>,
    movie_collection: web::Data<Collection<Movie>>,
) -> impl Responder {
    if let Ok(_claims) = verify(req).await {
        let filter = doc! { "_id": movie_id.into_inner() };
        match movie_collection.find_one(filter).await {
            Ok(Some(movie)) => HttpResponse::Ok().json(movie),
            Ok(None) => HttpResponse::NotFound().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}


// GET RANDOM MOVIE
pub async fn get_random_movie(
    req: HttpRequest,
    movie_collection: web::Data<Collection<Movie>>,
) -> impl Responder {
    let req_clone = req.clone();
    if let Ok(_claims) = verify(req_clone).await {
        let query_string = req.query_string();
        let filter = doc! { "is_series": query_string.contains("type=series") };
        let _options = mongodb::options::FindOptions::builder()
            .sort(doc! { "$sample": { "size": 1 } })
            .build();
        match movie_collection.find_one(filter).await {
            Ok(Some(movie)) => HttpResponse::Ok().json(movie),
            Ok(None) => HttpResponse::NotFound().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
