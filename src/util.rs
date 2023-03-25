#[derive(Debug, PartialEq)]
pub enum ValidationError {
    FieldToShort {
        field: String,
        min: usize,
        actual: usize,
    },
    FieldToLong {
        field: String,
        max: usize,
        actual: usize,
    },
}

pub fn validate_and_trim(
    name: String,
    value: String,
    min_len: usize,
    max_len: usize,
) -> Result<String, ValidationError> {
    let value = value.trim().to_string();
    if value.len() < min_len {
        return Err(ValidationError::FieldToShort {
            field: name,
            min: min_len,
            actual: value.len(),
        });
    }
    if value.len() > max_len {
        return Err(ValidationError::FieldToLong {
            field: name,
            max: max_len,
            actual: value.len(),
        });
    }
    Ok(value)
}
