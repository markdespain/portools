#[derive(Debug, PartialEq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    pub fn new(message: String) -> ValidationError {
        ValidationError { message }
    }
}

pub fn trim_and_validate_len(
    name: &str,
    value: String,
    min_len: usize,
    max_len: usize,
) -> Result<String, ValidationError> {
    let value = value.trim().to_string();
    let actual_len = value.len();
    if actual_len < min_len {
        return Err(ValidationError::new(format!(
            "field to short. field: {name}, min_len: {min_len}, actual_len: {actual_len}"
        )));
    }
    if actual_len > max_len {
        return Err(ValidationError::new(format!(
            "field to long. field: {name}, max_len: {max_len}, actual_len: {actual_len}"
        )));
    }
    Ok(value)
}
