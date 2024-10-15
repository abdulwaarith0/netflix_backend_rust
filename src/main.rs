#![allow(unused_imports)]

mod models;
mod routes;
mod utils;
mod verify_token;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::net::TcpListener;
use routes::auth::{login_user, register_user};
use routes::lists::{create_list, delete_list, get_lists};
use routes::movies::{create_movie, get_all_movies, get_movie, get_random_movie};
use routes::users::{get_all_users, get_user};

use std::env;

use crate::models::{list_mod, movie_mod, user_mod, users_mod};
use crate::routes::{auth, lists, movies, users};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv().ok();

    let mongodb_url = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    let client_options = ClientOptions::parse(&mongodb_url)
        .await
        .expect("Failed to parse MongoDB URL");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");
    let auth_db = client
        .database("test")
        .collection::<user_mod::User>("users");
    let movie_collection = client
        .database("test")
        .collection::<movie_mod::Movie>("movies");
    let list_collection = client
        .database("test")
        .collection::<list_mod::List>("lists");
    let users_collection = client
        .database("test")
        .collection::<users_mod::Users>("users");

    println!("MongoDB database connected!");

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("https://visionarynetflixclone.vercel.app/")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .app_data(web::Data::new(auth_db.clone()))
            .app_data(web::Data::new(movie_collection.clone()))
            .app_data(web::Data::new(list_collection.clone()))
            .app_data(web::Data::new(users_collection.clone()))
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(register_user))
                    .route("/login", web::post().to(login_user)),
            )
            .service(
                web::scope("/api/movies")
                    .route("/", web::post().to(create_movie))
                    .route("/", web::get().to(get_all_movies))
                    .route("/find/{id}", web::get().to(get_movie))
                    .route("/random", web::get().to(get_random_movie)),
            )
            .service(
                web::scope("/api/lists")
                    .route("/", web::post().to(create_list))
                    .route("/{id}", web::delete().to(delete_list))
                    .route("/", web::get().to(get_lists)),
            )
            .service(
                web::scope("/api/users")
                    .route("/", web::get().to(get_all_users))
                    .route("/{id}", web::get().to(get_user))
            )
    })
    .listen(listener)?
    ;

    println!("Server connected on port {}", port);

    server.run().await?;

    Ok(())
}
