use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt, TypedHeader,
};
use jsonwebtoken::{
    decode, encode, errors::Result as JwtResult, DecodingKey, EncodingKey, Header, Validation,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::utils::{get_epoch_ts, AppError};

lazy_static! {
    pub static ref JWT_KEYS: JwtKeys = JwtKeys::new();
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    fn new() -> Self {
        let secret = std::env::var("JWT_SECRET_KEY").unwrap_or("my_secret".to_string());
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, id: u32, name: Option<String>) -> JwtResult<String> {
        let jwt_expiry = std::env::var("JWT_EXPIRY").unwrap_or_default();
        let jwt_expiry = jwt_expiry.parse::<usize>().unwrap_or(3600);
        let jwt_expiry = get_epoch_ts() as usize + jwt_expiry;
        self.sign(id, name, jwt_expiry)
    }

    pub fn generate_refresh_token(&self, id: u32, name: Option<String>) -> JwtResult<String> {
        let jwt_expiry = std::env::var("REFRESH_TOKEN_EXPIRY").unwrap_or_default();
        let jwt_expiry = jwt_expiry.parse::<usize>().unwrap_or(24 * 3600);
        let jwt_expiry = get_epoch_ts() as usize + jwt_expiry;
        self.sign(id, name, jwt_expiry)
    }

    fn sign(&self, id: u32, name: Option<String>, exp: usize) -> JwtResult<String> {
        let claims = JwtClaims::new(id, name, exp);
        encode(&Header::default(), &claims, &self.encoding)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub exp: usize,
}

impl JwtClaims {
    fn new(id: u32, name: Option<String>, exp: usize) -> Self {
        Self { id, name, exp }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Auth("Missing token".into()))?;
        let token_data =
            decode::<JwtClaims>(bearer.token(), &JWT_KEYS.decoding, &Validation::default())
                .map_err(|_| AppError::Auth("Invalid Token".into()))?;
        Ok(token_data.claims)
    }
}
