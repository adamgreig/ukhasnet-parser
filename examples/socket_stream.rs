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
    nn: String,
    p: String,
    r: i32,
    t: String
}

fn main() {
    let stream = TcpStream::connect("ukhas.net:3020").unwrap();
    let mut bufstream = BufReader::new(stream);
    loop {
        let mut data = String::new();
        match bufstream.read_line(&mut data) {
            Ok(_) => (),
            Err(e) => {
                println!("Error reading from socket: {}", e);
                break
            }
        }

        let message = match json::decode::<SocketMessage>(&data) {
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
            Err(e) => { println!("Error parsing packet: {}", e); continue; },
        }

        println!("");

    }
}
