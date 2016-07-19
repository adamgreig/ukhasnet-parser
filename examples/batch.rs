/*
 * Batch process a text file full of packets, report how many parsed OK.
 */

extern crate ukhasnet_parser;
use ukhasnet_parser::{parse};
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::env::args;

fn main() {
    let f = match File::open(args().nth(1).unwrap()) {
        Ok(f) => f,
        Err(_) => panic!()
    };
    let r = BufReader::new(f);
    let mut total = 0;
    let mut parsed = 0;

    for line in r.lines() {
        let line = line.unwrap();
        total += 1;
        match parse(&line) {
            Ok(_) => parsed += 1,
            Err(_) => (),
        }
    }

    println!("Parsed {}/{} packets", parsed, total);
}
