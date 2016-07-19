#![recursion_limit = "1000"]

#[macro_use]
extern crate pest;
pub use pest::{Parser, StringInput};

mod parser;
mod packet;

pub use parser::{Rdp, ParserError};
pub use packet::{Location, WindSpeed, DataField, Packet};

pub fn parse(packet: &String) -> Result<Packet, ParserError> {
    let mut parser = Rdp::new(StringInput::new(packet));
    match parser.packet() {
        true => Ok(parser.parse()),
        false => Err(ParserError::from_parser(&mut parser)),
    }
}
