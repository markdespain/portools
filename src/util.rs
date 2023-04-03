#[derive(Debug, PartialEq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    pub fn new(message: String) -> ValidationError {
        ValidationError { message }
    }
}

pub fn validate_and_trim(
    name: String,
    value: String,
    min_len: usize,
    max_len: usize,
) -> Result<String, ValidationError> {
    let value = value.trim().to_string();
    let len = value.len();
    if len < min_len {
        return Err(ValidationError::new(format!(
            "filed to short. field: {name}, min_len: {min_len}, acual_len: {len}"
        )));
    }
    if len > max_len {
        return Err(ValidationError::new(format!(
            "filed to long. field: {name}, max_len: {max_len}, acual_len: {len}"
        )));
    }
    Ok(value)
}
