use chrono::ParseError;
use rust_decimal::Decimal;
use rusty_money::MoneyError;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct Invalid {
    pub field: String,
    pub reason: Reason,
}

impl Invalid {
    pub fn required(field: String) -> Invalid {
        Invalid {
            field,
            reason: Reason::Required,
        }
    }

    pub fn required_str(field: &str) -> Invalid {
        Invalid::required(field.to_string())
    }

    pub fn decoding_error(field: String, cause: &dyn Error) -> Invalid {
        Invalid {
            field,
            reason: Reason::DecodingError {
                cause: cause.to_string(),
            },
        }
    }

    pub fn unknown_error(field: &str, cause: &dyn Error) -> Invalid {
        Invalid {
            field: field.to_string(),
            reason: Reason::UnknownError {
                cause: cause.to_string(),
            },
        }
    }

    pub fn parse_decimal_error(field: &str, cause: rust_decimal::Error) -> Invalid {
        Invalid {
            field: field.to_string(),
            reason: Reason::ParseDecimalError { cause },
        }
    }

    pub fn parse_money_error(field: &str, cause: MoneyError) -> Invalid {
        Invalid {
            field: field.to_string(),
            reason: Reason::ParseMoneyError { cause },
        }
    }

    pub fn parse_date_error(field: &str, cause: ParseError) -> Invalid {
        Invalid {
            field: field.to_string(),
            reason: Reason::ParseDateError { cause },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Reason {
    Required,
    MustBePositive,
    MustHaveLongerLen,
    MustHaveShorterLen,
    DecodingError { cause: String },
    UnknownError { cause: String },
    ParseDecimalError { cause: rust_decimal::Error },
    ParseDateError { cause: ParseError },
    ParseMoneyError { cause: MoneyError },
}

pub fn validate_positive(field: &str, value: &Decimal) -> Result<(), Invalid> {
    if value.is_sign_negative() || value.is_zero() {
        return Err(Invalid {
            field: field.to_string(),
            reason: Reason::MustBePositive,
        });
    }
    Ok(())
}

pub fn trim_and_validate_len(
    field: &str,
    value: &str,
    min_len: usize,
    max_len: usize,
) -> Result<String, Invalid> {
    let value = value.trim();
    let len = value.len();
    if len < min_len {
        return Err(Invalid {
            field: field.to_string(),
            reason: Reason::MustHaveLongerLen,
        });
    }
    if len > max_len {
        return Err(Invalid {
            field: field.to_string(),
            reason: Reason::MustHaveShorterLen,
        });
    }
    Ok(value.to_string())
}
