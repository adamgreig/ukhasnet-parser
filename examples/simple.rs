extern crate ukhasnet_parser;

use ukhasnet_parser::parse;

fn main() {
    let s = "2bT12,15H38:test[AG]".to_owned();

    println!("Parsing {}...", s);

    match parse(&s) {
        Ok(p) => println!("{:?}", p),
        Err(_) => println!("Error"),
    }
}
