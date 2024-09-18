mod verify_token;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::env;

use crate::routes::{auth, lists, movies, users};
use crate::models::{user_mod, movie_mod, list_mod};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let mongodb_url = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    let client_options = ClientOptions::parse(&mongodb_url)
        .await
        .expect("Failed to parse MongoDB URL");

    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    println!("MongoDB database connected!");

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5174")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600),
            )
            .app_data(web::Data::new(client.clone()))
            
    })
    .bind(format!("localhost:{}", port))?;

    println!("Server connected on port {}", port);

    server.run().await?;

    Ok(())
}
