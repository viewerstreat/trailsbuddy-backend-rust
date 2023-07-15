use axum::{
    async_trait,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestExt,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use validator::{Validate, ValidationError};

use super::get_epoch_ts;

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

/// Custom validator function to check if provided timestamp value is future data
pub fn validate_future_timestamp(dt: &DateTime<Utc>) -> Result<(), ValidationError> {
    let curr_ts = get_epoch_ts() as i64;
    if dt.timestamp() <= curr_ts {
        let mut err = ValidationError::new("timestamp");
        err.message = Some(format!("timestamp must be future date").into());
        return Err(err);
    }
    Ok(())
}

/// custom validator function to check tags value
pub fn validate_tags(tags: &Vec<String>) -> Result<(), ValidationError> {
    if tags.iter().any(|tag| tag.is_empty()) {
        let mut err = ValidationError::new("tags");
        err.message = Some("empty tags are not allowed".into());
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
    // type Rejection = (StatusCode, Json<Value>);
    type Rejection = Response;

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        // extract the JSON body
        let Json(data) = req.extract::<Json<T>, _>().await.map_err(|err| {
            let msg = format!("Error extracting the JSON body");
            tracing::debug!(msg);
            err.into_response()
        })?;
        // validate json body
        data.validate().map_err(|err| {
            let msg = format!("Error validating json body: {err}");
            tracing::debug!(msg);
            let res = json!({"success": false, "message": msg});
            (StatusCode::BAD_REQUEST, Json(res)).into_response()
        })?;
        // return the validated body
        Ok(Self(data))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_validate_phonenumber() {
        let result = validate_phonenumber("");
        let result = result.err().unwrap();
        assert_eq!(result.code, "phone");
        assert_eq!(
            result.message,
            Some("Phone must be 10 digits. Invalid phone received: ".into())
        );

        let result = validate_phonenumber("1234");
        let result = result.err().unwrap();
        assert_eq!(result.code, "phone");
        assert_eq!(
            result.message,
            Some("Phone must be 10 digits. Invalid phone received: 1234".into())
        );
        let result = validate_phonenumber("123456789012");
        let result = result.err().unwrap();
        assert_eq!(result.code, "phone");
        assert_eq!(
            result.message,
            Some("Phone must be 10 digits. Invalid phone received: 123456789012".into())
        );

        let result = validate_phonenumber("abcdefghij");
        let result = result.err().unwrap();
        assert_eq!(result.code, "phone");
        assert_eq!(
            result.message,
            Some("Phone must be all digits. Invalid phone received: abcdefghij".into())
        );

        let result = validate_phonenumber("1234567890");
        assert_eq!(result.is_ok(), true);
    }
}
