#[macro_use]
extern crate nom;

mod parser;
pub use parser::{parse, Location, WindSpeed, DataField, Packet, Done, Error, Incomplete};
