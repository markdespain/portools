mod portfolio;
use chrono::naive::NaiveDate;

use crate::portfolio::Lot;

fn main() {
    let lot = Lot::new(
        String::from("Joint Taxable"),
        String::from("VOO"),
        NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
        6,
        321.55,
    );
    println!("Hello, lot! {:?}", &lot);
}
