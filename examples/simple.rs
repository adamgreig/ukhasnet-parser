extern crate ukhasnet_parser;
extern crate nom;

use ukhasnet_parser::parser::parse;
use nom::IResult::{Done, Error};

pub fn main() {
    let s = "2bT12,15H38:test[AG]";
    match parse(&s) {
        Done(_, p) => println!("{:?}", p),
        Error(e) => println!("Error {}", e),
        _ => println!("Unknown error")
    }
}
