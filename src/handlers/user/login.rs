use axum::{extract::State, Json};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mockall_double::double;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use validator::Validate;

use super::model::{LoginScheme, User};
use crate::{
    constants::*,
    jwt::JWT_KEYS,
    utils::{get_epoch_ts, get_seq_nxt_val, AppError, ValidatedBody},
};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
pub enum SocialLoginScheme {
    GOOGLE,
    FACEBOOK,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login_scheme: SocialLoginScheme,
    pub id_token: Option<String>,
    pub fb_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    success: bool,
    data: User,
    token: String,
}

#[derive(Debug)]
struct TokenData {
    name: String,
    email: String,
    profile_pic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdTokenClaims {
    sub: String,
    name: String,
    email: String,
    picture: Option<String>,
    iat: usize,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct JwkKeys {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct JwksResp {
    keys: Vec<JwkKeys>,
}

impl From<IdTokenClaims> for TokenData {
    fn from(value: IdTokenClaims) -> Self {
        Self {
            name: value.name.clone(),
            email: value.email.clone(),
            profile_pic: value.picture.clone(),
        }
    }
}

impl From<SocialLoginScheme> for LoginScheme {
    fn from(value: SocialLoginScheme) -> Self {
        match value {
            SocialLoginScheme::GOOGLE => Self::GOOGLE,
            SocialLoginScheme::FACEBOOK => Self::FACEBOOK,
        }
    }
}

pub async fn login_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<LoginRequest>,
) -> Result<Json<Response>, AppError> {
    let token_data = verify_token(&body).await?;
    let user = create_or_update_user(&db, &token_data, body.login_scheme.into()).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_owned()))?;
    let res = Response {
        success: true,
        data: user,
        token,
    };
    Ok(Json(res))
}

async fn verify_token(body: &LoginRequest) -> Result<TokenData, AppError> {
    match body.login_scheme {
        SocialLoginScheme::GOOGLE => {
            let id_token = body
                .id_token
                .as_ref()
                .ok_or(AppError::Auth("idToken missing".into()))?;
            Ok(verify_id_token(id_token).await?)
        }
        SocialLoginScheme::FACEBOOK => {
            let fb_token = body
                .fb_token
                .as_ref()
                .ok_or(AppError::Auth("fbToken missing".into()))?;
            Ok(verify_fb_token(fb_token).await?)
        }
    }
}

async fn verify_id_token(id_token: &str) -> Result<TokenData, AppError> {
    let jwks_resp = reqwest::get(GOOGLE_JWKS_URI)
        .await?
        .json::<JwksResp>()
        .await?;
    let parts = id_token.split(".").collect::<Vec<_>>();
    let Some(token_header) = parts.get(0) else {
        let err = AppError::Auth("Invalid token, could not split".into());
        return Err(err);
    };
    let bytes = engine::GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD)
        .decode(token_header)?;
    let token_header = serde_json::from_slice::<JsonValue>(&bytes)?;
    let kid = token_header["kid"]
        .as_str()
        .ok_or(AppError::Auth("Invalid token, could not get kid".into()))?;
    let idx = jwks_resp
        .keys
        .iter()
        .position(|k| k.kid.as_str() == kid)
        .ok_or(AppError::Auth("Invalid token, not valid kid".into()))?;
    let n = jwks_resp.keys[idx].n.as_str();
    let e = jwks_resp.keys[idx].e.as_str();
    let decoding_key =
        DecodingKey::from_rsa_components(n, e).map_err(|err| AppError::Auth(err.to_string()))?;
    let validation = Validation::new(Algorithm::RS256);
    let decoded_token = decode::<IdTokenClaims>(&id_token, &decoding_key, &validation)
        .map_err(|err| AppError::Auth(err.to_string()))?;
    Ok(decoded_token.claims.into())
}

async fn verify_fb_token(fb_token: &str) -> Result<TokenData, AppError> {
    let url = format!(
        "{}?access_token={}&fields=id,name,email,picture",
        FB_ME_URL, fb_token
    );
    let res = reqwest::get(&url).await?.json::<JsonValue>().await?;
    let name = res["name"]
        .as_str()
        .ok_or(AppError::Auth("Invalid token: name not found".into()))?;
    let email = res["email"]
        .as_str()
        .ok_or(AppError::Auth("Invalid token: email not found".into()))?;
    let profile_pic = res
        .get("picture")
        .and_then(|picture| picture.get("data"))
        .and_then(|data| data["url"].as_str())
        .and_then(|url| Some(url.to_string()));
    let token_data = TokenData {
        name: name.to_string(),
        email: email.to_string(),
        profile_pic,
    };

    Ok(token_data)
}

async fn create_or_update_user(
    db: &Arc<AppDatabase>,
    token_data: &TokenData,
    login_scheme: LoginScheme,
) -> Result<User, AppError> {
    let filter = Some(doc! {"email": token_data.email.as_str()});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if let Some(user) = user {
        if !user.is_active {
            let err = AppError::BadRequestErr("user is inactive".into());
            return Err(err);
        }
        update_user(db, user.id, login_scheme).await
    } else {
        create_user(db, token_data, login_scheme).await
    }
}

async fn update_user(
    db: &Arc<AppDatabase>,
    user_id: u32,
    login_scheme: LoginScheme,
) -> Result<User, AppError> {
    let filter = doc! {"id": user_id};
    let ts = get_epoch_ts() as i64;
    let login_scheme = login_scheme.to_string();
    let update = doc! {"$set": {"lastLoginTime": ts, "loginScheme": login_scheme}};
    let mut options = FindOneAndUpdateOptions::default();
    options.upsert = Some(false);
    options.return_document = Some(ReturnDocument::After);
    let options = Some(options);
    let user = db
        .find_one_and_update::<User>(DB_NAME, COLL_USERS, filter, update, options)
        .await?
        .ok_or(anyhow::anyhow!("Not able to update user"))?;
    Ok(user)
}

async fn create_user(
    db: &Arc<AppDatabase>,
    token_data: &TokenData,
    login_scheme: LoginScheme,
) -> Result<User, AppError> {
    let id = get_seq_nxt_val(USER_ID_SEQ, &db).await?;
    let ts = get_epoch_ts();
    let mut user = User::default();
    user.id = id;
    user.name = token_data.name.to_owned();
    user.email = Some(token_data.email.clone());
    user.profile_pic = token_data.profile_pic.clone();
    user.is_active = true;
    user.login_scheme = login_scheme;
    user.total_played = Some(0);
    user.contest_won = Some(0);
    user.total_earning = Some(0);
    user.created_ts = Some(ts);
    user.last_login_time = Some(ts);
    db.insert_one::<User>(DB_NAME, COLL_USERS, &user, None)
        .await?;
    Ok(user)
}
