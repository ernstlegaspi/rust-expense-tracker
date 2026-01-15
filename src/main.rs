mod handlers;
mod models;
mod routes;
mod services;

use actix_web::{App, HttpServer, Responder, get, web::Data};
use sqlx::postgres::PgPoolOptions;
use std::env::var;
use std::io::Result;

use crate::{
    routes::user_routes,
    services::{jwt_services::JwtService, user_services::UserService},
};

#[get("/test")]
async fn test_route() -> impl Responder {
    "A Test Route"
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let jwt_service = JwtService::new(jwt_secret);
    let user_service = UserService::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(user_service.clone()))
            .app_data(Data::new(jwt_service.clone()))
            .configure(user_routes::route)
            .service(test_route)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
