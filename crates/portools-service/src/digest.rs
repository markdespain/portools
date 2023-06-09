use actix_web::web::{Buf, Bytes};
use csv::StringRecord;
use portools_common::model::Lot;
use portools_common::validate::Invalid;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum CsvError {
    RecordError { row: usize, cause: String },
    RecordInvalid { row: usize, cause: Invalid },
    MissingHeader { name: String },
    HeaderError { cause: String },
}

pub fn csv_to_lot(csv: Bytes) -> Result<Vec<Lot>, CsvError> {
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let field_to_index = create_headers_to_index(rdr.headers())?;
    let mut lots = Vec::new();
    for (row, record) in rdr.records().enumerate() {
        match record {
            Ok(r) => match to_lot(row, &field_to_index, &r) {
                Ok(lot) => {
                    lots.push(lot);
                }
                Err(error) => {
                    return Err(error);
                }
            },
            Err(cause) => {
                return Err(CsvError::RecordError {
                    row,
                    cause: cause.to_string(),
                });
            }
        }
    }
    Ok(lots)
}

fn create_headers_to_index(
    headers: Result<&StringRecord, csv::Error>,
) -> Result<HashMap<String, usize>, CsvError> {
    let mut field_to_index = HashMap::with_capacity(5);
    let headers = headers.map_err(|error| CsvError::HeaderError {
        cause: error.to_string(),
    })?;
    for (i, header) in headers.iter().enumerate() {
        let header = header.trim().to_ascii_lowercase();
        field_to_index.insert(header, i);
    }
    Ok(field_to_index)
}

fn to_lot(
    row: usize,
    field_to_index: &HashMap<String, usize>,
    record: &StringRecord,
) -> Result<Lot, CsvError> {
    Lot::from_str(
        get_field(row, "account", field_to_index, record)?,
        get_field(row, "symbol", field_to_index, record)?,
        get_field(row, "date_acquired", field_to_index, record)?,
        get_field(row, "quantity", field_to_index, record)?,
        get_field(row, "cost_per_share", field_to_index, record)?,
    )
    .map_err(|cause| CsvError::RecordInvalid { row, cause })
}

fn get_field<'a>(
    row: usize,
    name: &'a str,
    name_to_index: &'a HashMap<String, usize>,
    record: &'a StringRecord,
) -> Result<&'a str, CsvError> {
    let index = name_to_index
        .get(name)
        .ok_or(CsvError::MissingHeader { name: name.into() })?;
    let field_value = record
        .get(*index)
        .ok_or(CsvError::RecordInvalid {
            row,
            cause: Invalid::required_str(name),
        })?
        .trim();
    Ok(field_value)
}

#[cfg(test)]
mod test {
    use crate::digest::{csv_to_lot, CsvError};
    use crate::unit_test_util::resource;
    use actix_web::web::Bytes;
    use chrono::NaiveDate;
    use portools_common::model::{Currency, Lot};
    use portools_common::validate::Invalid;
    use rust_decimal::Decimal;
    use rusty_money::MoneyError::InvalidAmount;
    use test_util::assertion::assert_err_eq;

    #[test]
    fn test_valid() {
        let csv = resource::load_bytes("valid.csv");
        let expected = vec![
            new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
            new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
            new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
        ];
        let result = csv_to_lot(Bytes::from(csv));
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_valid_different_column_order() {
        let csv = resource::load_bytes("valid_different_column_order.csv");
        let expected = vec![
            new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
            new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
            new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
        ];
        let result = csv_to_lot(Bytes::from(csv));
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_valid_with_whitespace() {
        let csv = resource::load_bytes("valid_with_whitespace.csv");
        let expected = vec![
            new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
            new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
            new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
        ];
        let result = csv_to_lot(Bytes::from(csv));
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_valid_with_capitalized_headers() {
        let csv = resource::load_bytes("valid_with_capitalized_headers.csv");
        let expected = vec![
            new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
            new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
            new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
        ];
        let result = csv_to_lot(Bytes::from(csv));
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_missing_header() {
        let csv = resource::load_bytes("missing_header.csv");
        let result = csv_to_lot(Bytes::from(csv));
        assert_err_eq(
            CsvError::MissingHeader {
                name: "account".into(),
            },
            result,
        );
    }

    #[test]
    fn test_missing_quantity_column() {
        let csv = resource::load_bytes("missing_quantity_column.csv");
        let result = csv_to_lot(Bytes::from(csv));
        assert_err_eq(
            CsvError::MissingHeader {
                name: "quantity".into(),
            },
            result,
        );
    }

    #[test]
    fn test_row_with_invalid_value() {
        let csv = resource::load_bytes("row_with_invalid_value.csv");
        let result = csv_to_lot(Bytes::from(csv));
        assert_err_eq(
            CsvError::RecordInvalid {
                row: 1,
                cause: Invalid::parse_money_error("cost_basis", InvalidAmount),
            },
            result,
        );
    }

    const DATE_FORMAT: &'static str = "%Y/%m/%d";

    fn new_lot(account: &str, symbol: &str, date: &str, quantity: u32, cost_basis_usd: f64) -> Lot {
        let cost_basis = Currency::new(cost_basis_usd.to_string().parse().unwrap(), "USD").unwrap();
        Lot::new(
            account,
            symbol,
            NaiveDate::parse_from_str(date, DATE_FORMAT).unwrap(),
            Decimal::from(quantity),
            cost_basis,
        )
        .unwrap()
    }
}
