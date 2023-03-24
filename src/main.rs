mod lot;

use crate::lot::Lot;

fn main() {
    let lot = Lot {
        account : String::from("Joint Taxable"),
        symbol : String::from("VOO"),
        date : String::from("2023/03/23"),
        quantity : 6,
        cost_basis : 321.55
    };
    println!("Hello, lot! {:?}", &lot);
}
