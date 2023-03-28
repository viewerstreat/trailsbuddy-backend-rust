use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::{constants::*, utils::get_epoch_ts};

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenClaims {
    iss: String,
    iat: u64,
    exp: u64,
    aud: String,
    scope: String,
}

impl GoogleTokenClaims {
    fn new() -> Self {
        let ts = get_epoch_ts();
        Self {
            iss: FIREBASE_SERVICE_CLIENT_EMAIL.to_string(),
            iat: ts,
            exp: ts + 3600,
            aud: GOOGLE_TOKEN_URL.to_string(),
            scope: FIREBASE_MESSAGE_SCOPE.to_string(),
        }
    }
}

#[derive(Default)]
pub struct GoogleAuthToken {
    access_token: Option<String>,
    valid_till: Option<u64>,
    signing_key: Option<EncodingKey>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: u64,
}

impl GoogleAuthToken {
    pub async fn get_access_token(&mut self) -> anyhow::Result<&str> {
        if self.is_new_token_required() {
            self.new_access_token().await?;
        }
        Ok(self.access_token.as_ref().unwrap())
    }

    fn get_signing_key(&mut self) -> anyhow::Result<&EncodingKey> {
        if self.signing_key.is_none() {
            let key = EncodingKey::from_rsa_pem(FIREBASE_SERVICE_PRIVATE_KEY.as_bytes())?;
            self.signing_key = Some(key);
        }
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or(anyhow::anyhow!("signing_key not found"))?;
        Ok(signing_key)
    }

    fn is_new_token_required(&self) -> bool {
        if let Some(valid_till) = self.valid_till {
            let ts = get_epoch_ts();
            return ts <= valid_till;
        }
        self.access_token.is_none() || self.valid_till.is_none()
    }

    async fn new_access_token(&mut self) -> anyhow::Result<()> {
        let signed_jwt = self.new_jwt()?;
        let bearer_token = format!("Bearer {}", &signed_jwt);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, bearer_token.as_str().parse()?);
        headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", signed_jwt.as_str()),
        ];
        let client = reqwest::Client::new();
        let response = client
            .post(GOOGLE_TOKEN_URL)
            .headers(headers)
            .form(&params)
            .send()
            .await?
            .json::<GoogleTokenResponse>()
            .await?;
        let ts = get_epoch_ts();
        let valid_till = response.expires_in + ts - (15 * 60);
        self.access_token = Some(response.access_token);
        self.valid_till = Some(valid_till);

        Ok(())
    }

    fn new_jwt(&mut self) -> anyhow::Result<String> {
        let claims = GoogleTokenClaims::new();
        let key = self.get_signing_key()?;
        let header = Header::new(Algorithm::RS256);
        let jwt = encode(&header, &claims, &key)?;
        Ok(jwt)
    }
}
