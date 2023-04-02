use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::http::header::CONTENT_LENGTH;
use actix_web::HttpRequest;
use futures_util::TryStreamExt;
use ContentLengthHeaderError::MalformedContentLengthHeader;

pub fn get_content_length_header(
    req: &HttpRequest,
) -> actix_web::Result<usize, ContentLengthHeaderError> {
    let header_value = req
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or(ContentLengthHeaderError::NoContentLengthHeader)?;
    let header_str = header_value.to_str().map_err(|e| {
        MalformedContentLengthHeader(format!("failed to convert content-length to a str: {e}"))
    })?;
    let content_length = header_str.parse().map_err(|e| {
        MalformedContentLengthHeader(format!("failed to parse content-length to to a u8: {e}"))
    })?;
    if content_length > 0 {
        Ok(content_length)
    } else {
        Err(MalformedContentLengthHeader(format!(
            "content-length was not positive: {content_length}"
        )))
    }
}

#[derive(Debug)]
pub enum ContentLengthHeaderError {
    NoContentLengthHeader,
    MalformedContentLengthHeader(String),
}

pub async fn field_to_vec(
    field: &mut Field,
    max_num_bytes: usize,
    init_capacity_num_bytes: usize,
) -> actix_web::Result<Vec<u8>, UploadError> {
    // todo: is there a better way to buffer?
    let mut csv_bytes: Vec<u8> = Vec::with_capacity(init_capacity_num_bytes);
    loop {
        match field.try_next().await {
            Ok(None) => {
                break;
            }
            Ok(Some(chunk)) => {
                if csv_bytes.len() + max_num_bytes > max_num_bytes {
                    return Err(UploadError::MaxSizeExceeded);
                }
                csv_bytes.append(&mut chunk.to_owned().to_vec());
            }
            Err(e) => {
                return Err(UploadError::MultipartError(e));
            }
        }
    }
    Ok(csv_bytes)
}

pub async fn multipart_to_vec(
    payload: &mut Multipart,
    max_num_bytes: usize,
    init_capacity_num_bytes: usize,
) -> actix_web::Result<Vec<u8>, UploadError> {
    match payload.try_next().await {
        Ok(Some(mut field)) => {
            field_to_vec(&mut field, max_num_bytes, init_capacity_num_bytes).await
        }
        Ok(None) => Err(UploadError::NoFile),
        Err(e) => Err(UploadError::MultipartError(e)),
    }
}

#[derive(Debug)]
pub enum UploadError {
    NoFile,
    MaxSizeExceeded,
    MultipartError(MultipartError),
}
