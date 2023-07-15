use axum::{extract::State, Json};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

/// Add user view for movie
#[utoipa::path(
    post,
    path = "/api/v1/movie/addView",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = MovieAddViewReqBody,
    responses(
        (status = StatusCode::OK, description = "Successful", body = AddViewResponse),
        (status = StatusCode::NOT_FOUND, description = "movie not found", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Invalid token", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn add_movie_view_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<MovieAddViewReqBody>,
) -> Result<Json<AddViewResponse>, AppError> {
    let ts = get_epoch_ts();
    let view_entry = ViewsEntry {
        user_id: claims.id,
        updated_ts: Some(ts),
    };
    let movie_id = parse_object_id(&body.movie_id, "not able to parse movie_id")?;
    let filter = Some(doc! {"_id": movie_id.clone()});
    let movie = db
        .find_one::<Movie>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("Movie not found".into()))?;
    if let Some(views) = &movie.props.views {
        if views.iter().any(|v| v.user_id == claims.id) {
            let view_count = views.len() as u32;
            let res = AddViewResponse {
                success: true,
                message: "User already viewed".to_string(),
                view_count,
            };
            return Ok(Json(res));
        }
    };
    let find_by = doc! {"_id": movie_id, "views.userId": {"$ne": claims.id}};
    let update = doc! {"$set": {"updatedTs": ts as i64, "updatedBy": claims.id}, "$push": {"views": view_entry}};
    let mut options = FindOneAndUpdateOptions::default();
    options.return_document = Some(ReturnDocument::After);
    let options = Some(options);
    let result = db
        .find_one_and_update::<Movie>(DB_NAME, COLL_MOVIES, find_by, update, options)
        .await?
        .ok_or(anyhow::anyhow!("Not able to update any document"))?;
    let view_count = result
        .props
        .views
        .and_then(|view| Some(view.len() as u32))
        .unwrap_or_default();

    let res = AddViewResponse {
        success: true,
        message: "Updated successfully".to_string(),
        view_count,
    };
    Ok(Json(res))
}
