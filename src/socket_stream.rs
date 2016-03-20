extern crate ukhasnet_parser;
extern crate rustc_serialize;
extern crate nom;

use std::str;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

use rustc_serialize::json;
use ukhasnet_parser::{parse, Done, Error, Incomplete};

#[derive(Debug,RustcDecodable)]
struct SocketMessage {
    i: u32,
    ni: u32,
    nn: String,
    p: String,
    r: i32,
    s: String,
    t: String
}

fn main() {
    let stream = TcpStream::connect("ukhas.net:3010").unwrap();
    let mut bufstream = BufReader::new(stream);
    loop {
        let mut data = Vec::new();
        match bufstream.read_until(b'}', &mut data) {
            Ok(_) => (),
            Err(e) => {
                println!("Error reading from socket: {}", e);
                break
            }
        }

        let jsonstr = match str::from_utf8(&data) {
            Ok(s) => s,
            Err(e) => {
                println!("Error converting data to string: {}", e);
                continue;
            }
        };

        let message = match json::decode::<SocketMessage>(&jsonstr) {
            Ok(m) => m,
            Err(e) => {
                println!("Error parsing message JSON: {}", e);
                continue;
            }
        };

        println!("[{}] ({}) {}:", message.t, message.r, message.nn);

        match parse(&message.p) {
            Done(_, p) => println!("{:?}", p),
            Error(e) => {println!("Error parsing packet: {}", e); continue;},
            Incomplete(_) => {println!("Incomplete data"); continue;}
        }

        println!("");
    }
}
