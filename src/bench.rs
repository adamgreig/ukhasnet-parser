extern crate ukhasnet_parser;
use ukhasnet_parser::{parse, Done, Error, Incomplete};
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::path::Path;
use std::env::args;
use std::string::String;

fn main() {
    let f = match File::open(args().nth(1).unwrap()) {
        Ok(f) => f,
        Err(e) => panic!()
    };
    let r = BufReader::new(f);
    for line in r.lines() {
        let line = line.unwrap();
        parse(&line);
    }
}
