use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::{collections::HashMap, sync::Arc};

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, get_random_num, AppError, ValidatedBody},
};

/// Create a broadcast push notification
#[utoipa::path(
    post,
    path = "/api/v1/notification/createBroadcast",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = CreateBroadcastReq,
    responses(
        (status = StatusCode::OK, description = "Saved successfully", body = GenericResponse),
    ),
    tag = "Admin API"
)]
pub async fn create_broadcast_noti_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateBroadcastReq>,
) -> Result<Json<GenericResponse>, AppError> {
    let ts = get_epoch_ts();
    let random = get_random_num(101, 1000);
    let event_name = format!("BROADCAST_MESSAGE_{ts}_{random}");
    let content = NotificationContent {
        event_name: event_name.to_owned(),
        content: body.message.to_owned(),
        created_by: Some(claims.data.id),
        created_ts: Some(ts),
    };
    db.insert_one::<NotificationContent>(DB_NAME, COLL_NOTIFICATION_CONTENTS, &content, None)
        .await?;
    let filter = doc! {"isActive": true};
    let users = db
        .find::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await?;

    let requests = users
        .into_iter()
        .map(|user| NotificationReq::new(user.id, &event_name, HashMap::new()))
        .collect::<Vec<_>>();
    if !requests.is_empty() {
        db.insert_many::<NotificationReq>(DB_NAME, COLL_NOTIFICATION_REQUESTS, &requests, None)
            .await?;
    }

    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}
