use crate::config::Claims;
use crate::request_inputs::CreateUserInput;
use crate::request_outputs::{CreateUserOutput, SignInOutput};
use jsonwebtoken::{EncodingKey, Header, encode};
use poem::Error;
use poem::http::StatusCode;
use poem::{
    EndpointExt, handler,
    web::{Data, Json},
};
use std::sync::{Arc, Mutex};
use store::store::Store;

#[handler]
pub async fn sign_in(
    Json(data): Json<CreateUserInput>,
    Data(s): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<SignInOutput>, poem::Error> {
    let mut locked_s = s.lock().unwrap();

    let exists = locked_s.sign_in(data.email.clone(), data.password.clone());
    match (exists) {
        Ok(user_id) => {
            let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
                Error::from_string("JWT_SECRET not set", StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
            let my_claims = Claims {
                sub: user_id,
                exp: 10000000000,
            };
            let token = encode(&Header::default(), &my_claims, &encoding_key).map_err(|_| {
                Error::from_string("Token generation failed", StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            let response = SignInOutput { jwt: token };
            Ok(Json(response))
        }
        Err(e) => {
            return Err(Error::from_string(
                "Invalid Credentials",
                StatusCode::UNAUTHORIZED,
            ));
        }
    }
}

#[handler]
pub async fn sign_up(
    Json(data): Json<CreateUserInput>,
    Data(s): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<CreateUserOutput>, poem::Error> {
    let mut locked_s = s.lock().unwrap();
    let user = locked_s.sign_up(data.email, data.password);

    match (user) {
        Ok(user_id) => {
            let response: CreateUserOutput = CreateUserOutput { user_id };
            Ok(Json(response))
        }
        Err(e) => {
            return Err(Error::from_string(
                "Email already exists",
                StatusCode::CONFLICT,
            ));
        }
    }
}
