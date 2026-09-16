use crate::config::Claims;
use jsonwebtoken::{DecodingKey, Validation, decode};
use poem::http::header::AUTHORIZATION;
use poem::{Error, FromRequest, Request, RequestBody, Result, http::StatusCode};

pub struct UserId(pub String);

impl<'a> FromRequest<'a> for UserId {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(
                std::env::var("JWT_SECRET")
                    .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
                    .as_bytes(),
            ),
            &Validation::default(),
        )
        .map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;

        Ok(UserId(token_data.claims.sub))
    }
}
