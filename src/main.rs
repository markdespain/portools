mod lot;

use crate::lot::Lot;

fn main() {
    let lot = Lot::new(
        String::from("Joint Taxable"),
        String::from("VOO"),
        String::from("2023/03/23"),
        6,
        321.55,
    );
    println!("Hello, lot! {:?}", &lot);
}
