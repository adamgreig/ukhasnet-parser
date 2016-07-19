extern crate ukhasnet_parser;
extern crate pest;
use ukhasnet_parser::{Rdp, StringInput, Parser};
use std::env::args;

fn main() {
    let packet = if args().len() == 2 {
        args().nth(1).unwrap()
    } else {
        "2bT12.34,15H38W123Z1:hello world[AG]".to_owned()
    };
    println!("Parsing '{}':", packet);

    let mut parser = Rdp::new(StringInput::new(&packet));
    match parser.packet() {
        true => println!("Parsed OK:"),
        false => {
            let (expected, position) = parser.expected();

            println!("Failure at input position {}", position);
            println!("\n{}", packet);
            for _ in 0..position { print!(" "); }
            println!("^");

            println!("Expected one of:");
            println!("{:?}", expected);

            panic!("Cannot proceed");
        }
    };

    let mut indent = 0;
    let mut parents = vec![&parser.queue()[0]];
    let mut prev_t = parents[0];

    for t in parser.queue() {
        if t.start >= prev_t.start && t.end <= prev_t.end {
            indent += 1;
            parents.push(prev_t);
        } else {
            while t.start >= parents.last().unwrap().end {
                parents.pop();
                indent -= 1;
            }
        }
        for _ in 0..(indent-1) {
            print!("    ");
        }
        println!("{:?} {}-{}", t.rule, t.start, t.end);
        prev_t = t;
    }
    println!("");

    println!("{:?}", parser.parse());
}
