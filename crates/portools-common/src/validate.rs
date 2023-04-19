use chrono::ParseError;
use rust_decimal::Decimal;
use rusty_money::MoneyError;

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
        Invalid::required(field.into())
    }

    pub fn parse_decimal_error(field: &str, cause: rust_decimal::Error) -> Invalid {
        Invalid {
            field: field.into(),
            reason: Reason::ParseDecimalError { cause },
        }
    }

    pub fn parse_money_error(field: &str, cause: MoneyError) -> Invalid {
        Invalid {
            field: field.into(),
            reason: Reason::ParseMoneyError { cause },
        }
    }

    pub fn parse_date_error(field: &str, cause: ParseError) -> Invalid {
        Invalid {
            field: field.into(),
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
    ParseDecimalError { cause: rust_decimal::Error },
    ParseDateError { cause: ParseError },
    ParseMoneyError { cause: MoneyError },
}

pub fn validate_positive(field: &str, value: &Decimal) -> Result<(), Invalid> {
    if value.is_sign_negative() || value.is_zero() {
        return Err(Invalid {
            field: field.into(),
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
            field: field.into(),
            reason: Reason::MustHaveLongerLen,
        });
    }
    if len > max_len {
        return Err(Invalid {
            field: field.into(),
            reason: Reason::MustHaveShorterLen,
        });
    }
    Ok(value.into())
}
