use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::{bson::doc, options::FindOptions};
use std::sync::Arc;

use crate::{constants::*, database::AppDatabase, jwt::JwtClaims, models::*, utils::AppError};

/// get notifications
#[utoipa::path(
    get,
    path = "/api/v1/notification",
    params(GetNotiReq, ("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "notification list", body = GetNotiResp),
    ),
    tag = "App User API"
)]
pub async fn get_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<GetNotiReq>,
) -> Result<Json<GetNotiResp>, AppError> {
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let skip = page_index * page_size;
    let sort = doc! {"_id": -1};
    let mut options = FindOptions::default();
    options.sort = Some(sort);
    options.skip = Some(skip);
    options.limit = Some(page_size as i64);
    let options = Some(options);
    let push_message = NotificationType::PUSH_MESSAGE.to_bson()?;
    let filter = doc! {"userId": claims.id, "isCleared": false, "notificationType": push_message};
    let result = db
        .find::<Notifications>(DB_NAME, COLL_NOTIFICATIONS, Some(filter), options)
        .await?;
    let res = GetNotiResp {
        success: true,
        data: result,
    };
    Ok(Json(res))
}
