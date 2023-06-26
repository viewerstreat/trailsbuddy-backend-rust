use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    models::movie::MovieDetails,
    utils::{parse_object_id, AppError},
};

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
    let oid = parse_object_id(&params.movie_id, "invalid movieId")?;
    let filter = Some(doc! {"_id": oid});
    let mut movie = db
        .find_one::<MovieDetails>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("movie not found".into()))?;
    movie.view_count = get_view_count(&movie);
    movie.like_count = get_like_count(&movie);
    let res = Response {
        success: true,
        data: movie,
    };
    Ok(Json(res))
}

fn get_view_count(movie: &MovieDetails) -> Option<u32> {
    movie
        .props
        .views
        .as_ref()
        .and_then(|views| Some(views.len() as u32))
}

fn get_like_count(movie: &MovieDetails) -> Option<u32> {
    movie.props.likes.as_ref().and_then(|likes| {
        let count = likes
            .iter()
            .map(|like| if like.is_removed { 0 } else { 1 })
            .sum::<u32>();
        Some(count)
    })
}
