extern crate ukhasnet_parser;

use ukhasnet_parser::{parse};

pub fn main() {
    let s = "2bT12,15H38:test[AG]".to_owned();
    match parse(&s) {
        Some(p) => println!("{:?}", p),
        None => println!("Error"),
    }
}
