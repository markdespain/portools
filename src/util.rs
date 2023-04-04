use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    pub fn new(message: String) -> ValidationError {
        ValidationError { message }
    }
}

pub fn validate_positive(name: &str, value: &Decimal) -> Result<(), ValidationError> {
    if value.is_sign_negative() || value.is_zero() {
        return Err(ValidationError::new(format!(
            "field must be positive. field: {name}, value: {value}"
        )));
    }
    Ok(())
}

pub fn trim_and_validate_len(
    name: &str,
    value: &str,
    min_len: usize,
    max_len: usize,
) -> Result<String, ValidationError> {
    let value = value.trim();
    let len = value.len();
    if len < min_len {
        return Err(ValidationError::new(format!(
            "field to short. field: {name}, min_len: {min_len}, len: {len}"
        )));
    }
    if len > max_len {
        return Err(ValidationError::new(format!(
            "field to long. field: {name}, max_len: {max_len}, len: {len}"
        )));
    }
    Ok(value.to_string())
}
