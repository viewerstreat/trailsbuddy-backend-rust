use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use mockall_double::double;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    constants::*,
    utils::{get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody},
};

#[double]
use crate::database::AppDatabase;

#[double]
use crate::utils::misc_outer::misc_inner;

use super::{model::User, otp::generate_send_otp};

fn call_add_one(n: u32) -> u32 {
    misc_inner::add_one(n)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
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
    check_uniq_phone(&db, body.phone.as_str()).await?;
    // check if email already exists in the DB
    if let Some(email) = &body.email {
        check_uniq_email(&db, email.as_str()).await?;
    }
    let user = body.create_user(&db).await?;
    db.insert_one::<User>(DB_NAME, COLL_USERS, &user, None)
        .await?;
    // generate and send otp to the phone
    generate_send_otp(user.id, &db).await?;
    // return successful response
    let response = (
        StatusCode::CREATED,
        Json(json!({"success": true, "message": "User created"})),
    );
    Ok(response)
}

// check if the given phone already exists in users collection
async fn check_uniq_phone(db: &Arc<AppDatabase>, phone: &str) -> Result<(), AppError> {
    let filter = Some(doc! {"phone": phone});
    let result = db
        .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if result.is_some() {
        let err = format!("User already exists with same phone: {}", phone);
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }

    Ok(())
}

// check if the given email already exists in the users collection
async fn check_uniq_email(db: &Arc<AppDatabase>, email: &str) -> Result<(), AppError> {
    let filter = Some(doc! {"email": email});
    let result = db
        .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if result.is_some() {
        let err = format!("User already exists with same email: {}", email);
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use mockall::predicate::{eq, function};
    use mongodb::options::FindOneOptions;
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
    async fn test_check_uniq_phone() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_phone(&db, phone).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_phone_exists() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_phone(&db, phone).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same phone: {}", phone);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
        }
    }

    #[tokio::test]
    async fn test_check_uniq_email() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_email(&db, email).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_email_exists() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_email(&db, email).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same email: {}", email);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
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
    }

    #[derive(Debug, Deserialize)]
    struct Response {
        success: bool,
        message: String,
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
            let req = Request::builder()
                .uri("/")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap();
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
            let req = Request::builder()
                .uri("/")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap();
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
        {
            let app = app.clone();
            create_user_req.phone = "000111222234".to_string();
            let body = serde_json::to_string(&create_user_req).unwrap();
            let req = Request::builder()
                .uri("/")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap();
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
            let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
            let response: Response = serde_json::from_slice(&bd).unwrap();
            assert_eq!(response.success, false);
        }
    }

    #[test]
    fn test_call_add_one() {
        let ctx = misc_inner::add_one_context();
        ctx.expect().with(eq(5)).times(1).returning(|_| 2);
        assert_eq!(call_add_one(5), 2);
    }
}
