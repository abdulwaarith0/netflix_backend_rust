#![allow(unused_imports)]

mod models;
mod routes;
mod utils;
mod verify_token;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dotenv::dotenv;
use env_logger::Env;
use mongodb::{options::ClientOptions, Client};
use std::net::TcpListener;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use routes::auth::{login_user, register_user};
use routes::lists::{create_list, delete_list, get_lists};
use routes::movies::{create_movie, get_all_movies, get_movie, get_random_movie};
use routes::users::{get_all_users, get_user};

use crate::models::{list_mod, movie_mod, user_mod, users_mod};
use crate::routes::{auth, lists, movies, users};


// Handler for the health check endpoint
async fn health_check() -> impl Responder {
    log::info!("Health check endpoint was accessed");
    HttpResponse::Ok().json("Service is up and running")
}

// main function to run the server 
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set the logging level from the environment variable
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    // Set the logging level from the environment variable or use debug as default
    let logging_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    // Set the max level based on the logging level
    let max_level = match logging_level.as_str() {
        "trace" => Level::TRACE,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        _ => Level::INFO,
    };

    // Create a subscriber that logs to the console with the max level 
    let subscriber = FmtSubscriber::builder()
        .with_max_level(max_level)
        .finish();
    
    // Set the subscriber as the global default
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    info!("Starting server...");
    // Load the environment variables from the .env file
    dotenv().ok();

    // Get the MongoDB URL from the environment variable 
    let mongodb_url = env::var("MONGODB_URL").expect("MONGODB_URL must be set");

    // Parse the MongoDB URL and create a client 
    let client_options = ClientOptions::parse(&mongodb_url)
        .await
        .expect("Failed to parse MongoDB URL");

    // Create a client with the client_options 
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    // Get the database and collections
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

    info!("MongoDB database connected!");  

    // Get the port from the environment variable or use 8080 as default
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Bind the server to the port using the TcpListener
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();

    // Create the server with the routes
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                // Allow the frontend to access the server
                Cors::default()
                    .allowed_origin("https://visionarynetflixclone.vercel.app")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600),
            )
            // Log the request details
            .wrap(Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
            .app_data(web::Data::new(auth_db.clone()))
            .app_data(web::Data::new(movie_collection.clone()))
            .app_data(web::Data::new(list_collection.clone()))
            .app_data(web::Data::new(users_collection.clone()))
            .service(
                // Define the routes for the authentication
                web::scope("/api/auth")
                    .route("/register", web::post().to(register_user))
                    .route("/login", web::post().to(login_user)),
            )
            .service(
                // Define the routes for the movies
                web::scope("/api/movies")
                    .route("/", web::post().to(create_movie))
                    .route("/", web::get().to(get_all_movies))
                    .route("/find/{id}", web::get().to(get_movie))
                    .route("/random", web::get().to(get_random_movie)),
            )
            .service(
                // Define the routes for the lists
                web::scope("/api/lists")
                    .route("/", web::post().to(create_list))
                    .route("/{id}", web::delete().to(delete_list))
                    .route("/", web::get().to(get_lists)),
            )
            .service(
                // Define the routes for the users
                web::scope("/api/users")
                    .route("/", web::get().to(get_all_users))
                    .route("/{id}", web::get().to(get_user))
            )
            .service(
                // Handler for the health check endpoint
                web::scope("/api/health")
                    .route("/", web::get().to(health_check))
            )
    })
    .listen(listener)?
    ;

    info!("Server connected on port {}", port);

    // Run the server
    server.run().await?;

    Ok(())
}
