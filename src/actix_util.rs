use actix_web::http::header::CONTENT_LENGTH;
use actix_web::HttpRequest;
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
