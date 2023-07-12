use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    models::*,
    utils::{parse_object_id, AppError},
};

/// Get movie details
///
/// Get details of a movie
#[utoipa::path(
    get,
    path = "/api/v1/movie/details",
    params(MovieDetailParams),
    responses(
        (status = StatusCode::OK, description = "Movie details", body = MovieDetailResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn movie_details_handler(
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<MovieDetailParams>,
) -> Result<Json<MovieDetailResponse>, AppError> {
    let oid = parse_object_id(&params.movie_id, "invalid movieId")?;
    let filter = Some(doc! {"_id": oid});
    let mut movie = db
        .find_one::<MovieDetails>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("movie not found".into()))?;
    movie.view_count = get_view_count(&movie);
    movie.like_count = get_like_count(&movie);
    let res = MovieDetailResponse {
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
