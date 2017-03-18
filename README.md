# ukhasnet-parser

A parser for the UKHASnet protocol written in Rust using the Nom library.

```rust
extern crate ukhasnet_parser;

use ukhasnet_parser::{parse};

pub fn main() {
    let s = "2bT12,15H38:test[AG]".to_owned();
    match parse(&s) {
        Some(p) => println!("{:?}", p),
        None => println!("Error"),
    }
}
```

```sh
$ ./target/debug/ukhasnet-parser-simple-demo
Packet { repeat: 2, sequence: 'b', data: [Temperature([12, 15]), Humidity([38])], comment: Some("test"), path: ["AG"] }
```
