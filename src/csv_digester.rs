use crate::model::Lot;
use crate::validate::Invalid;
use actix_web::web::{Buf, Bytes};
use csv::StringRecord;
use std::collections::HashMap;
use uuid::Uuid;

pub fn csv_to_lot(csv: Bytes) -> Result<Vec<Lot>, Invalid> {
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let mut field_to_index = HashMap::with_capacity(5);
    let headers = rdr
        .headers()
        .map_err(|error| Invalid::unknown_error("csv_headers", &error))?;
    for (i, header) in headers.iter().enumerate() {
        field_to_index.insert(header.to_owned(), i);
    }
    if headers.is_empty() {
        return Err(Invalid::required_str("csv_headers"));
    }
    let mut lots = Vec::new();
    for (row, record) in rdr.records().enumerate() {
        match record {
            Ok(r) => match to_lot(&field_to_index, &r) {
                Ok(lot) => {
                    lots.push(lot);
                }
                Err(e) => {
                    println!("failed to convert record to Lot: {:?}", e);
                    return Err(e);
                }
            },
            Err(cause) => {
                return Err(Invalid::decoding_error(format!("row {row}"), &cause));
            }
        }
    }
    Ok(lots)
}

fn to_lot(field_to_index: &HashMap<String, usize>, record: &StringRecord) -> Result<Lot, Invalid> {
    Lot::from_str(
        Uuid::new_v4(),
        get_field("account", field_to_index, record)?,
        get_field("symbol", field_to_index, record)?,
        get_field("date_acquired", field_to_index, record)?,
        get_field("quantity", field_to_index, record)?,
        get_field("cost_per_share", field_to_index, record)?,
    )
}

fn get_field<'a>(
    field: &'a str,
    field_to_index: &'a HashMap<String, usize>,
    record: &'a StringRecord,
) -> Result<&'a str, Invalid> {
    let field_index = field_to_index
        .get(field)
        .ok_or(Invalid::required(format!("header: {:?}", field)))?;
    let field_value = record
        .get(*field_index)
        .ok_or(Invalid::required_str(field))?;
    Ok(field_value)
}

#[cfg(test)]
mod test {
    use crate::csv_digester::csv_to_lot;
    use crate::model::{Currency, Lot};
    use crate::test_util;
    use crate::test_util::assertion::{assert_err_eq, assert_vec_eq_fn};
    use actix_web::web::Bytes;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::fs;
    use std::path::PathBuf;
    use test_util::fixture;
    use uuid::Uuid;
    use crate::validate::Invalid;

    fn load_resource(resource: &str) -> Bytes {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resource/test/csv_digester/");
        path.push(resource);
        let p = path.display();
        println!("path: {p}");
        Bytes::from(fs::read(path).unwrap())
    }

    #[test]
    fn test_valid() {
        let csv = load_resource("valid.csv");
        let expected = vec![
            new_lot("Taxable", "VOO", 27, 1, 100.47),
            new_lot("IRA", "BND", 28, 2, 200.26),
            new_lot("IRA", "BND", 29, 3, 300.23),
        ];
        let eq_ignore_id = |a: &Lot, b: &Lot| a.eq_ignore_id(b);
        let result = csv_to_lot(Bytes::from(csv));
        assert_vec_eq_fn(&expected, &result, eq_ignore_id);
    }

    #[test]
    fn test_missing_header() {
        let csv = load_resource("missing_header.csv");
        let result = csv_to_lot(Bytes::from(csv));
        assert_err_eq(Invalid::required_str("header: \"account\""), result);
    }

    #[test]
    fn test_missing_quantity_column() {
        let csv = load_resource("missing_quantity_column.csv");
        let result = csv_to_lot(Bytes::from(csv));
        assert_err_eq(Invalid::required_str("header: \"quantity\""), result);
    }

    fn new_lot(
        account: &str,
        symbol: &str,
        day_of_month: u32,
        quantity: u32,
        cost_basis_usd: f64,
    ) -> Lot {
        let cost_basis = Currency::new(cost_basis_usd.to_string().parse().unwrap(), "USD").unwrap();
        Lot::new(
            Uuid::new_v4(),
            account,
            symbol,
            NaiveDate::from_ymd_opt(2023, 3, day_of_month).unwrap(),
            Decimal::from(quantity),
            cost_basis,
        )
        .unwrap()
    }
}
