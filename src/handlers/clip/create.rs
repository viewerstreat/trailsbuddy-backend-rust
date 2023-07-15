use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

/// Clip create
///
/// Create a new clip
#[utoipa::path(
    post,
    path = "/api/v1/clip",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = CreateClipReqBody,
    responses(
        (status = StatusCode::OK, description = "Clip created", body = ClipResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn create_clip_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateClipReqBody>,
) -> Result<Json<ClipResponse>, AppError> {
    let claims = claims.data;
    check_duplicate_name(&db, &body.name).await?;
    let ts = get_epoch_ts();
    let clip_props = ClipProps {
        name: body.name,
        description: body.description,
        banner_image_url: body.banner_image_url,
        video_url: body.video_url,
        is_active: true,
        likes: Some(vec![]),
        views: Some(vec![]),
    };
    let clip = Clips {
        props: clip_props,
        created_by: Some(claims.id),
        created_ts: Some(ts),
        updated_by: None,
        updated_ts: None,
    };
    let result = db
        .insert_one::<Clips>(DB_NAME, COLL_CLIPS, &clip, None)
        .await?;
    let res = ClipResponse {
        success: true,
        data: clip.to_clip_resp_data(&result),
    };
    Ok(Json(res))
}

async fn check_duplicate_name(db: &Arc<AppDatabase>, name: &str) -> Result<(), AppError> {
    let filter = doc! {"name": name};
    let clip = db
        .find_one::<Clips>(DB_NAME, COLL_CLIPS, Some(filter), None)
        .await?;
    if clip.is_some() {
        let err = "Clip exists with same name";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    Ok(())
}
