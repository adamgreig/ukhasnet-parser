# ukhasnet-parser

A parser for the UKHASnet protocol written in Rust.

See the `examples` directory for further usage, or simply:

```rust
extern crate ukhasnet_parser;

use ukhasnet_parser::{parse};

pub fn main() {
    let s = "2bT12,15H38:test[AG]".to_owned();
    match parse(&s) {
        Ok(p) => println!("{:?}", p),
        Err(e) => println!("{}", e),
    }
}
```

```sh
$ cargo run --example simple
Packet { repeat: 2, sequence: 'b', data: [Temperature([12, 15]), Humidity([38])], comment: Some("test"), path: ["AG"] }
```
