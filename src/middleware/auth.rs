use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized, web};
use std::future::{Ready, ready};

use crate::services::jwt_services::JwtService;

pub struct AuthMiddleware {
    pub uuid: uuid::Uuid,
}

impl FromRequest for AuthMiddleware {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = || -> Result<Self, Error> {
            let jwt = req
                .app_data::<web::Data<JwtService>>()
                .ok_or_else(|| ErrorUnauthorized("Jwt Service not configured."))?;

            let cookie = req
                .cookie("token")
                .ok_or_else(|| ErrorUnauthorized("Missing token"))?;

            let claims = jwt
                .validate_token(cookie.value())
                .map_err(|_| ErrorUnauthorized("Invalid token"))?;

            Ok(AuthMiddleware { uuid: claims.sub })
        };

        ready(result())
    }
}
