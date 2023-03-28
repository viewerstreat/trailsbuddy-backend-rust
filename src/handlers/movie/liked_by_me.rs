use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{constants::*, jwt::JwtClaims, models::movie::Movie, utils::AppError};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    movie_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    success: bool,
    is_liked_by_me: bool,
}

pub async fn is_liked_by_me_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<Params>,
) -> Result<Json<Response>, AppError> {
    let oid = ObjectId::parse_str(params.movie_id.as_str()).map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("invalid movieId".into())
    })?;
    let filter = Some(doc! {"_id": oid});
    let movie = db
        .find_one::<Movie>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("movie not found".into()))?;
    let is_liked_by_me = movie
        .likes
        .and_then(|likes| {
            let is_liked = likes
                .iter()
                .any(|like| like.user_id == claims.id && like.is_removed == false);
            Some(is_liked)
        })
        .unwrap_or(false);
    let res = Response {
        success: true,
        is_liked_by_me,
    };
    Ok(Json(res))
}
