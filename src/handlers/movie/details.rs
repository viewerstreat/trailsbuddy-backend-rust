use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{constants::*, models::movie::MovieDetails, utils::AppError};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    movie_id: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: MovieDetails,
}

pub async fn movie_details_handler(
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<Params>,
) -> Result<Json<Response>, AppError> {
    let oid = ObjectId::parse_str(params.movie_id.as_str()).map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("invalid movieId".into())
    })?;
    let filter = Some(doc! {"_id": oid});
    let mut movie = db
        .find_one::<MovieDetails>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("movie not found".into()))?;
    let view_count = movie
        .views
        .as_ref()
        .and_then(|views| Some(views.len() as u32));
    let like_count = movie.likes.as_ref().and_then(|likes| {
        let count = likes
            .iter()
            .map(|like| if like.is_removed { 0 } else { 1 })
            .sum::<u32>();
        Some(count)
    });
    movie.view_count = view_count;
    movie.like_count = like_count;
    let res = Response {
        success: true,
        data: movie,
    };
    Ok(Json(res))
}
