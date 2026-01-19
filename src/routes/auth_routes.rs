use actix_web::web::{ServiceConfig, post, scope};

use crate::handlers::auth::{login, register};

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/users")
            .route("/register", post().to(register))
            .route("/login", post().to(login)),
    );
}
