use crate::model::Lot;
use crate::validate::Invalid;
use actix_web::web::{Buf, Bytes};
use csv::StringRecord;
use std::collections::HashMap;

pub fn csv_to_lot(csv: Bytes) -> Result<Vec<Lot>, Invalid> {
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let mut field_to_index = HashMap::with_capacity(5);
    if !rdr.has_headers() {
        return Err(Invalid::required_str("csv_headers"));
    }
    let headers = rdr
        .headers()
        .map_err(|error| Invalid::unknown_error("csv_headers", &error))?;
    for (i, header) in headers.iter().enumerate() {
        field_to_index.insert(header.to_owned(), i);
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
        .ok_or(Invalid::required(format!("column: {:?}", field)))?;
    let field_value = record
        .get(*field_index)
        .ok_or(Invalid::required_str(field))?;
    Ok(field_value)
}
