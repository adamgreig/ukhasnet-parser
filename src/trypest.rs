extern crate ukhasnet_parser;
extern crate pest;
use ukhasnet_parser::Rdp;
//use ukhasnet_parser::pestpacket::{Packet,Location,WindSpeed,DataField};
use pest::prelude::*;

fn main() {
    let packet = "2bT12.34,15H38W123Z1:test[AG]";
    let mut parser = Rdp::new(StringInput::new(&packet));
    assert!(parser.packet());
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
