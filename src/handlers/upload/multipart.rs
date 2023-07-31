use aws_sdk_s3::model::{CompletedMultipartUpload, CompletedPart};
use axum::{
    extract::{Multipart, Query},
    Json,
};
use validator::Validate;

use crate::{
    constants::*,
    models::*,
    utils::{AppError, ValidatedBody},
};

/// Initiate multipart upload
#[utoipa::path(
    get,
    path = "/api/v1/upload/multipart/initiate",
    params(MultipartUploadInitiateReq),
    responses(
        (status = StatusCode::OK, description = "Multipart upload initiated", body = MultipartUploadInitiateRes),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn create_multipart_handler(
    Query(param): Query<MultipartUploadInitiateReq>,
) -> Result<Json<MultipartUploadInitiateRes>, AppError> {
    param
        .validate()
        .map_err(|err| AppError::BadRequestErr(err.to_string()))?;
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let key = super::uniq_file_name(&param.file_name);
    let res = client
        .create_multipart_upload()
        .bucket(AWS_BUCKET)
        .key(&key)
        .send()
        .await
        .map_err(|err| {
            tracing::debug!("{:?}", err);
            let err = anyhow::anyhow!(err.to_string());
            AppError::AnyError(err)
        })?;
    let upload_id = res
        .upload_id()
        .ok_or(AppError::AnyError(anyhow::anyhow!("upload_id not present")))?;
    let res = MultipartUploadInitiateRes {
        success: true,
        key,
        upload_id: upload_id.to_string(),
    };
    Ok(Json(res))
}

/// Multipart upload a part
///
/// `uploadId`, `partNumber` & `key` are mandatory parameters to be passed as query params
/// multipart request body will contain the chunk of the file.
#[utoipa::path(
    post,
    path = "/api/v1/upload/multipart/uploadPart",
    params(MultipartUploadPartReq),
    responses(
        (status = StatusCode::OK, description = "Multipart upload part successfull", body = UploadPartRes),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn upload_part_multipart_handler(
    Query(param): Query<MultipartUploadPartReq>,
    mut files: Multipart,
) -> Result<Json<UploadPartRes>, AppError> {
    param
        .validate()
        .map_err(|err| AppError::BadRequestErr(err.to_string()))?;
    let file = files
        .next_field()
        .await?
        .ok_or(AppError::BadRequestErr("no file".into()))?;
    let data = file.bytes().await.map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("unable to read file content".into())
    })?;
    let size = data.len();
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let res = client
        .upload_part()
        .key(&param.key)
        .bucket(AWS_BUCKET)
        .upload_id(&param.upload_id)
        .body(data.into())
        .part_number(param.part_number)
        .send()
        .await
        .map_err(|err| AppError::AnyError(anyhow::anyhow!(err.to_string())))?;

    let e_tag = res
        .e_tag()
        .ok_or(AppError::AnyError(anyhow::anyhow!("not able to get e_tag")))?;
    let completed_part = UploadPartCompleted {
        e_tag: e_tag.to_owned(),
        part_number: param.part_number,
        size,
    };
    let message = if size < MULTIPART_CHUNK_MIN_SIZE {
        Some(format!("Warning: Only the last chunk can have size less than {MULTIPART_CHUNK_MIN_SIZE}, otherwise whole upload will fail"))
    } else {
        None
    };
    let res = UploadPartRes {
        success: true,
        completed_part,
        message,
    };
    Ok(Json(res))
}

/// Finish multipart upload
///
/// Array of completed parts with eTag & partNumber are required
/// uploadId & key are also required
#[utoipa::path(
    post,
    path = "/api/v1/upload/multipart/finish",
    request_body = CompleteMultipartUploadReq,
    responses(
        (status = StatusCode::OK, description = "Multipart upload part successfull", body = UploadPartRes),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn complete_multipart_handler(
    ValidatedBody(body): ValidatedBody<CompleteMultipartUploadReq>,
) -> Result<Json<FileUploadRes>, AppError> {
    validate_finish_request_body(&body)?;
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let upload_parts = body
        .completed_parts
        .into_iter()
        .map(|part| {
            CompletedPart::builder()
                .e_tag(&part.e_tag)
                .part_number(part.part_number)
                .build()
        })
        .collect::<Vec<_>>();
    let completed_parts = CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts))
        .build();
    let res = client
        .complete_multipart_upload()
        .bucket(AWS_BUCKET)
        .key(body.key.as_str())
        .upload_id(body.upload_id.as_str())
        .multipart_upload(completed_parts)
        .send()
        .await
        .map_err(|err| {
            tracing::debug!("{:?}", err);
            AppError::AnyError(anyhow::anyhow!(err.to_string()))
        })?;
    let url = res.location().unwrap_or_default();
    let e_tag = res.e_tag().unwrap_or_default();
    let res = FileUploadRes {
        success: true,
        e_tag: e_tag.to_owned(),
        url: url.to_owned(),
    };
    Ok(Json(res))
}

fn validate_finish_request_body(body: &CompleteMultipartUploadReq) -> Result<(), AppError> {
    if body.completed_parts.is_empty() {
        let err = "completedParts is required";
        return Err(AppError::BadRequestErr(err.into()));
    }
    Ok(())
}
