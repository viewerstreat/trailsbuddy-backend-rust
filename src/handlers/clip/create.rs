use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
    jwt::JwtClaims,
    models::clip::{ClipRespData, Clips},
    utils::{AppError, ValidatedBody},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateClipReqBody {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(length(min = 1))]
    description: String,
    #[validate(url)]
    banner_image_url: String,
    #[validate(url)]
    video_url: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: ClipRespData,
}

pub async fn create_clip_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateClipReqBody>,
) -> Result<Json<Response>, AppError> {
    let clip = Clips::new(
        &body.name,
        &body.description,
        &body.banner_image_url,
        &body.video_url,
        claims.id,
    );
    let result = db
        .insert_one::<Clips>(DB_NAME, COLL_CLIPS, &clip, None)
        .await?;
    let res = Response {
        success: true,
        data: clip.to_clip_resp_data(&result),
    };
    Ok(Json(res))
}
