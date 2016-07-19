#![recursion_limit = "1000"]

#[macro_use]
extern crate pest;
pub use pest::Parser;
use pest::StringInput;

mod parser;
mod packet;

pub use parser::{Rdp};
pub use packet::{Location, WindSpeed, DataField, Packet};

pub fn parse(packet: &String) -> Option<Packet> {
    let mut parser = Rdp::new(StringInput::new(packet));
    match parser.packet() {
        true => Some(parser.parse()),
        false => None,
    }
}
