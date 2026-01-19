use actix_web::web::{ServiceConfig, post, scope};

use crate::handlers::auth::{login, refresh, register};

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/api/user")
            .route("/register", post().to(register))
            .route("/login", post().to(login))
            .route("/refresh", post().to(refresh)),
    );
}
