use crate::portfolio::Lot;
use crate::util::ValidationError;
use actix_web::web::{Buf, Bytes};
use csv::StringRecord;
use std::collections::HashMap;

pub fn csv_to_lot(csv: Bytes) -> Result<Vec<Lot>, ValidationError> {
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let mut field_to_index = HashMap::with_capacity(5);
    let headers = rdr
        .headers()
        .map_err(|error| ValidationError::new(format!("failed get headers: {:?}", error)))?;
    for (i, header) in headers.iter().enumerate() {
        field_to_index.insert(header.to_owned(), i);
    }
    let mut lots = Vec::new();
    for record in rdr.records() {
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
            Err(e) => {
                return Err(ValidationError::new(format!(
                    "failed to convert uploaded bytes to utf8: {e}"
                )));
            }
        }
    }
    Ok(lots)
}

fn to_lot(
    field_to_index: &HashMap<String, usize>,
    record: &StringRecord,
) -> Result<Lot, ValidationError> {
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
) -> Result<&'a str, ValidationError> {
    let field_index = field_to_index
        .get(field)
        .ok_or(ValidationError::new(format!("missing header: {:?}", field)))?;
    let field_value = record
        .get(*field_index)
        .ok_or(ValidationError::new(format!("missing value: {:?}", field)))?;
    Ok(field_value)
}
