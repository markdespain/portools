use actix_web::HttpRequest;
use actix_web::http::header::CONTENT_LENGTH;
use actix_multipart::{Field, Multipart};
use ContentLengthHeaderError::MalformedContentLengthHeader;
use futures_util::TryStreamExt;

pub fn get_content_length_header(req: &HttpRequest) -> actix_web::Result<usize, ContentLengthHeaderError> {
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

pub async fn field_to_vec(field: &mut Field, max_num_bytes: usize) -> actix_web::Result<Vec<u8>, UploadError> {
    let mut csv_bytes: Vec<u8> = Vec::new();
    while let Ok(Some(chunk)) = field.try_next().await {
        if csv_bytes.len() + max_num_bytes > max_num_bytes {
            return Err(UploadError::MaxSizeExceeded);
        }
        csv_bytes.append(&mut chunk.to_owned().to_vec());
    }
    Ok(csv_bytes)
}

pub async fn multipart_to_vec(
    payload: &mut Multipart,
    max_num_bytes: usize,
) -> actix_web::Result<Vec<u8>, UploadError> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        return field_to_vec(&mut field, max_num_bytes).await;
    }
    Err(UploadError::NoFile)
}

#[derive(Debug)]
pub enum UploadError {
    NoFile,
    MaxSizeExceeded,
}
