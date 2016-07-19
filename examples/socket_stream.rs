extern crate ukhasnet_parser;
extern crate rustc_serialize;

use std::str;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

use rustc_serialize::json;
use ukhasnet_parser::{parse};

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

        print!("[{}] ({}) {}: ", message.t, message.r, message.nn);
        println!("{}", message.p);

        match parse(&message.p) {
            Ok(p) => println!("{:?}", p),
            Err(_) => { println!("Error parsing packet"); continue; },
        }

        println!("");

    }
}
