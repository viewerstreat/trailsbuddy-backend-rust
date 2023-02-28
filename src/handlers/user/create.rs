use axum::{extract::State, http::StatusCode, Json};
use mockall_double::double;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use super::model::User;
use crate::{
    constants::*,
    utils::{get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody},
};

#[double]
use crate::database::AppDatabase;

#[double]
use super::helper::helper_inner;

#[double]
use super::otp::otp_inner;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(email)]
    email: Option<String>,

    #[serde(rename = "profilePic")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    profile_pic: Option<String>,
}

impl CreateUserReq {
    async fn create_user(&self, db: &Arc<AppDatabase>) -> anyhow::Result<User> {
        let id = get_seq_nxt_val(USER_ID_SEQ, db).await?;
        let mut user = User::default();
        user.id = id;
        user.name = self.name.to_owned();
        user.phone = self.phone.to_owned();
        user.email = self.email.clone();
        user.is_active = true;
        user.total_played = Some(0);
        user.contest_won = Some(0);
        user.total_earning = Some(0);
        user.created_ts = Some(get_epoch_ts());
        Ok(user)
    }
}

pub async fn create_user_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateUserReq>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // check if phone already exists in the DB
    helper_inner::check_uniq_phone(&db, body.phone.as_str()).await?;
    // check if email already exists in the DB
    if let Some(email) = &body.email {
        helper_inner::check_uniq_email(&db, email.as_str()).await?;
    }
    let user = body.create_user(&db).await?;
    db.insert_one::<User>(DB_NAME, COLL_USERS, &user, None)
        .await?;
    // generate and send otp to the phone
    otp_inner::generate_send_otp(user.id, &db).await?;
    // return successful response
    let response = (
        StatusCode::CREATED,
        Json(json!({"success": true, "message": "User created"})),
    );
    Ok(response)
}

/**
*
*
*
*
*/

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use mockall::predicate::{eq, function};
    use mongodb::{bson::Document, options::InsertOneOptions};
    use tower::ServiceExt;

    use super::*;

    fn get_test_create_user_req() -> CreateUserReq {
        CreateUserReq {
            name: "Test User".to_string(),
            phone: "1234567890".to_string(),
            email: Some("testemail@internet.org".to_string()),
            profile_pic: None,
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let user_id = 32;
        let seq_id = USER_ID_SEQ;
        let filter = doc! {"_id": seq_id};
        let update = doc! {"$inc": {"val": 1}};
        let alway_true = function(|_| true);
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one_and_update::<Document>()
            .with(
                eq(DB_NAME),
                eq(COLL_SEQUENCES),
                eq(filter),
                eq(update),
                alway_true,
            )
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(doc! {"val": user_id})));
        let db = Arc::new(mock_db);
        let create_user_req = get_test_create_user_req();
        let user = create_user_req.create_user(&db).await.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.name, create_user_req.name);
        assert_eq!(user.phone, create_user_req.phone);
        assert_eq!(user.email, create_user_req.email);
        assert_eq!(user.profile_pic, create_user_req.profile_pic);
        assert_eq!(user.is_active, true);
        assert_eq!(user.contest_won, Some(0));
        assert_eq!(user.total_played, Some(0));
        assert_eq!(user.total_earning, Some(0));
        assert_eq!(user.created_ts.is_some(), true);
    }

    #[derive(Debug, Deserialize)]
    struct Response {
        success: bool,
        message: String,
    }

    fn build_req(body: String) -> Request<String> {
        Request::builder()
            .uri("/")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_user_handler_empty_body() {
        let mock_db = AppDatabase::default();
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", post(create_user_handler))
            .with_state(db);
        let req = Request::builder()
            .uri("/")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, false);
    }

    #[tokio::test]
    async fn test_create_user_handler_invalid_name() {
        let mock_db = AppDatabase::default();
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", post(create_user_handler))
            .with_state(db);
        let mut create_user_req = get_test_create_user_req();
        {
            let app = app.clone();
            create_user_req.name = String::new();
            let body = serde_json::to_string(&create_user_req).unwrap();
            let req = build_req(body);
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
            let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
            let response: Response = serde_json::from_slice(&bd).unwrap();
            assert_eq!(response.success, false);
        }

        {
            let app = app.clone();
            create_user_req.name = (0..51).map(|_| 'a').collect();
            let body = serde_json::to_string(&create_user_req).unwrap();
            let req = build_req(body);
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
            let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
            let response: Response = serde_json::from_slice(&bd).unwrap();
            assert_eq!(response.success, false);
        }
    }

    #[tokio::test]
    async fn test_create_user_handler_invalid_phone() {
        let mock_db = AppDatabase::default();
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", post(create_user_handler))
            .with_state(db);
        let mut create_user_req = get_test_create_user_req();
        create_user_req.phone = "000111222234".to_string();
        let body = serde_json::to_string(&create_user_req).unwrap();
        let req = build_req(body);
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, false);
    }

    #[tokio::test]
    async fn test_create_user_handler_valid() {
        let uniq_phone_ctx = helper_inner::check_uniq_phone_context();
        let uniq_email_ctx = helper_inner::check_uniq_email_context();
        let otp_ctx = otp_inner::generate_send_otp_context();
        uniq_phone_ctx.expect().times(1).returning(|_, _| Ok(()));
        uniq_email_ctx.expect().times(1).returning(|_, _| Ok(()));
        otp_ctx.expect().times(1).returning(|_, _| Ok(()));
        let user_id = 23;
        let body = get_test_create_user_req();
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one_and_update::<Document>()
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(doc! {"val": user_id})));
        let is_none = function(|opt: &Option<InsertOneOptions>| opt.is_none());
        let check_user = {
            let body = body.clone();
            function(move |user: &User| {
                user.id == user_id
                    && user.name == body.name
                    && user.phone == body.phone
                    && user.email == body.email
                    && user.profile_pic == body.profile_pic
            })
        };
        mock_db
            .expect_insert_one::<User>()
            .with(eq(DB_NAME), eq(COLL_USERS), check_user, is_none)
            .times(1)
            .returning(|_, _, _, _| Ok(String::new()));

        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", post(create_user_handler))
            .with_state(db);
        let body = serde_json::to_string(&body).unwrap();
        let req = build_req(body);
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.message.as_str(), "User created");
    }
}
