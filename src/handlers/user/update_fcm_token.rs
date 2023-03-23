use axum::{extract::State, Json};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::model::User;
use crate::{
    constants::*,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
pub struct ReqBody {
    #[validate(length(min = 1))]
    token: String,
}

pub async fn update_fcm_token_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let filter = doc! {"id": claims.id};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter.clone()), None)
        .await?;
    let fcm_tokens = user.and_then(|user| user.fcm_tokens);
    if let Some(token) = fcm_tokens {
        if token
            .iter()
            .any(|token| token.as_str() == body.token.as_str())
        {
            let res = json!({"success": true, "message": "token already exists for user"});
            return Ok(Json(res));
        }
    }
    let ts = get_epoch_ts() as i64;
    let update = doc! {"$set": {"updatedTs": ts}, "$push": {"fcmTokens": &body.token}};
    let result = db
        .update_one(DB_NAME, COLL_USERS, filter, update, None)
        .await?;
    if result.matched_count < 1 || result.modified_count < 1 {
        let err = anyhow::anyhow!("not able to update user");
        return Err(AppError::AnyError(err));
    }

    let res = json!({"success": true, "message": "token saved successfully"});
    Ok(Json(res))
}
