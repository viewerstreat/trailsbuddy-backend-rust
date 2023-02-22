use axum::{
    async_trait,
    extract::FromRequest,
    http::{Request, StatusCode},
    Json, RequestExt,
};

use serde_json::{json, Value};
use validator::{Validate, ValidationError};

/// Custom validator function to check phone number
pub fn validate_phonenumber(phone: &str) -> Result<(), ValidationError> {
    // phone must be 10 digits long
    if phone.len() != 10 {
        let mut err = ValidationError::new("phone");
        err.message =
            Some(format!("Phone must be 10 digits. Invalid phone received: {phone}").into());
        return Err(err);
    }
    // phone must be all numeric chars
    if !phone.chars().all(|ch| ch.is_ascii_digit()) {
        let mut err = ValidationError::new("phone");
        err.message =
            Some(format!("Phone must be all digits. Invalid phone received: {phone}").into());
        return Err(err);
    }

    Ok(())
}

pub struct ValidatedBody<T>(pub T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for ValidatedBody<T>
where
    B: Send + 'static,
    S: Send + Sync,
    T: Validate + 'static,
    Json<T>: FromRequest<(), B>,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        // extract the JSON body
        let Json(data) = req.extract::<Json<T>, _>().await.map_err(|_| {
            let msg = format!("Error extracting the JSON body");
            tracing::debug!(msg);
            let res = json!({"success": false, "message": msg});
            (StatusCode::BAD_REQUEST, Json(res))
        })?;
        // validate json body
        data.validate().map_err(|err| {
            let msg = format!("Error validating json body: {}", err);
            tracing::debug!(msg);
            let res = json!({"success": false, "message": msg});
            (StatusCode::BAD_REQUEST, Json(res))
        })?;
        // return the validated body
        Ok(Self(data))
    }
}
