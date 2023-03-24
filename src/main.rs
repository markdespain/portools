mod portfolio;
use chrono::naive::NaiveDate;

use crate::portfolio::{Currency, Lot};

fn main() {
    let lot = Lot::new(
        String::from("Joint Taxable"),
        String::from("VOO"),
        NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
        6,
        Currency::new(30064, String::from("USD")).unwrap(),
    );
    println!("Hello, lot! {:?}", &lot);
}
