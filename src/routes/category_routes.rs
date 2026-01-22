use actix_web::web::{ServiceConfig, post, scope};

use crate::handlers::category::add_category;

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(scope("/api/category").route("/", post().to(add_category)));
}
