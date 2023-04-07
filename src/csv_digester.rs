use crate::portfolio::Lot;
use crate::util::Invalid;
use actix_web::web::{Buf, Bytes};
use csv::StringRecord;
use std::collections::HashMap;

pub fn csv_to_lot(csv: Bytes) -> Result<Vec<Lot>, Invalid> {
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let mut field_to_index = HashMap::with_capacity(5);
    let headers = rdr
        .headers()
        .map_err(|error| Invalid::format_error("headers", &error))?;
    for (i, header) in headers.iter().enumerate() {
        field_to_index.insert(header.to_owned(), i);
    }
    let mut lots = Vec::new();
    for (row, record) in rdr.records().enumerate() {
        match record {
            Ok(r) => match to_lot(&field_to_index, &r) {
                Ok(lot) => {
                    // todo: apply reasonable, configurable limits to the Lot values
                    // e.g. date shouldn't be in the ancient past
                    //      quantity shouldn't be absurdly high
                    //      cost_basis shouldn't be absurdly high
                    //      (quantity * cost_basis) shouldn't be absurdly high
                    //      total portfolio value shouldn't be absurdly high
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
