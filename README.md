# ukhasnet-parser

A parser for the UKHASnet protocol written in Rust using the Nom library.

```rust
extern crate ukhasnet_parser;
extern crate nom;

use ukhasnet_parser::parser::parse;
use nom::IResult::{Done, Error};

pub fn main() {
    let s = "2bT12,15H38:test[AG]";
    match parse(&s) {
        Done(_, p) => println!("{:?}", p),
        Error(e) => println!("Error {}", e),
        _ => println!("Unknown error")
    }
}
```

```sh
$ ./target/debug/ukhasnet-parser-simple-demo
Packet { repeat: 2, sequence: 'b', data: [Temperature([12, 15]), Humidity([38]), Comment("test")], path: ["AG"] }
```
