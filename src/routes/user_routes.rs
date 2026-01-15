use actix_web::web;

use crate::handlers::user::create_user;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").route("/", web::post().to(create_user)));
}
