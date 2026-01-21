mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use actix_web::{App, HttpServer, Responder, get, web::Data};
use sqlx::postgres::PgPoolOptions;
use std::env::var;
use std::io::Result;

use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    routes::auth_routes,
    services::{
        auth_services::AuthService, jwt_services::JwtService, redis_services::RedisService,
    },
};

#[get("/health")]
async fn health() -> impl Responder {
    "App is healthy and working."
}

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().compact().pretty())
        .init();

    dotenv::dotenv().ok();

    info!("Starting server...");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let redis_url = var("REDIS_URL").expect("REDIS_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let jwt_service = JwtService::new(jwt_secret);
    let auth_service = AuthService::new(pool);
    let redis_service = RedisService::new(redis_url.as_str()).expect("Failed to connect to Redis");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(Data::new(auth_service.clone()))
            .app_data(Data::new(jwt_service.clone()))
            .app_data(Data::new(redis_service.clone()))
            .configure(auth_routes::route)
            .service(health)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
