mod models;
mod routes;
mod utils;
mod verify_token;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger::Env;
use mongodb::{options::ClientOptions, Client};
use std::env;
use std::net::TcpListener;

use crate::models::{list, movie, user, users}; 
use routes::auth::{login_user, register_user};
use routes::lists::{create_list, delete_list, get_lists};
use routes::movies::{create_movie, get_all_movies, get_movie, get_random_movie};
use routes::users::{get_all_users, get_user};

// ── Health check ──────────────────────────────────────────────────────────────

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("Service is up and running")
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Single logging setup via env_logger.
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv().ok();

    // ── MongoDB ───────────────────────────────────────────────────────────────

    let mongodb_url = env::var("MONGODB_URL").expect("MONGODB_URL must be set");

    let client_options = ClientOptions::parse(&mongodb_url)
        .await
        .expect("Failed to parse MongoDB URL");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("test");

    // auth routes use the User model (register / login)
    let auth_collection = db.collection::<user::User>("users");
    let movie_collection = db.collection::<movie::Movie>("movies");
    let list_collection = db.collection::<list::List>("lists");
    // user management routes use the richer Users model (get / list)
    let users_collection = db.collection::<users::Users>("users");

    log::info!("MongoDB connected!");

    // ── Bind ──────────────────────────────────────────────────────────────────

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .unwrap_or_else(|_| panic!("Failed to bind to port {}", port));

    log::info!("Server listening on port {}", port);

    // ── Server ────────────────────────────────────────────────────────────────

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("https://visionarynetflixclone.vercel.app")
                    .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600),
            )
            .wrap(Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
            .app_data(web::Data::new(auth_collection.clone()))
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
                    .route("/{id}", web::get().to(get_user)),
            )
            .service(
                web::scope("/api/health")
                    .route("/", web::get().to(health_check)),
            )
    })
    .listen(listener)?
    .run()
    .await
}