extern crate ukhasnet_parser;
extern crate nom;
use std::env::args;
use nom::IResult::{Done, Error};

extern crate websocket;
extern crate hyper;
extern crate cookie;

fn main() {
    //match args().len() {
        //2 => {
            //let packet = args().nth(1).unwrap();
            //match ukhasnet_parser::parser::parse(&packet) {
                //Done(_, p) => println!("{:?}", p),
                //Error(e) => println!("Parse error: {:?}", e),
                //_ => println!("Unable to parse.")
            //}
        //},
        //_ => println!("Usage: {} <UKHASnet packet>", args().nth(0).unwrap())
    //}

    use std::thread;
    use std::sync::mpsc::channel;
    use std::io;
    use std::io::{stdin};
    use std::str::from_utf8;

    use websocket::{Message, Sender, Receiver};
    use websocket::message::Type;
    use websocket::client::request::Url;
    use websocket::Client;
    use websocket::header::Origin;

    use hyper;
    use hyper::header::{Headers, Cookie, SetCookie};
    use cookie::Cookie as CookiePair;


    let client = hyper::client::Client::new();
    let mut res = client.get("https://ukhas.net/socket.io/?EIO=3&transport=polling&t=100").send().unwrap();
    println!("Initial HTTP req.\n Status: {:?}\n Headers: {:?}\n Version: {:?}\n Body: ", res.status, res.headers, res.version);
    io::copy(&mut res, &mut io::stdout()).unwrap();
    println!("\n");

    let cookies = match res.headers.get::<SetCookie>() {
        Some(c) => {println!("Cookie: {:?}", c); c},
        None => {println!("No cookie found"); panic!()}
    };
    let sid = &cookies.iter().nth(0).unwrap().value;
    println!("SID: {:?}", sid);

    let mut res = client.post(&format!("https://ukhas.net/socket.io/?EIO=3&transport=polling&t=101&sid={}", sid)).body("10:40/logtail").header(Cookie(vec![CookiePair::new("io".to_owned(), sid.to_owned())])).send().unwrap();
    println!("POST: Status {:?} Headers {:?} Body ", res.status, res.headers);
    io::copy(&mut res, &mut io::stdout()).unwrap();
    println!("\n");

    let urlstr = format!("wss://ukhas.net/socket.io/?EIO=3&transport=websocket&sid={}", sid);
    let url = Url::parse(&urlstr).unwrap();


    println!("Connecting to {}", url);

    let mut request = Client::connect(url).unwrap();

    request.headers.set(
        Cookie(vec![
            CookiePair::new("io".to_owned(), sid.to_owned())
        ])
    );

    let origin = Origin("https://ukhas.net".to_string());
    request.headers.set(origin);

    println!("Request headers: {:?}\n", request.headers);

    let mut response = request.send().unwrap(); // Send the request and retrieve a response

    println!("Validating response...");

    println!("Response: Status {:?}\n Headers {:?}\n Version {:?}\n", response.status, response.headers, response.version);

    response.validate().unwrap(); // Validate the response

    println!("Successfully connected");

    println!("Status: {:?}\nHeaders: {:?}\n", &response.status, &response.headers);

    let (mut sender, mut receiver) = response.begin().split();

    //match sender.send_dataframe(&Message::binary(vec!{50, 112, 114, 111, 98, 101})) {
        //Ok(_) => println!("Sent OK"),
        //Err(e) => println!("Error: {:?}", e)
    //};
    //match sender.send_dataframe(&Message::binary(vec!{53})) {
        //Ok(_) => println!("Sent OK"),
        //Err(e) => println!("Error: {:?}", e)
    //};

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
        loop {
            // Send loop
            let message: Message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    return;
                }
            };
            match message.opcode {
                Type::Close => {
                    let _ = sender.send_message(&message);
                    // If it's a close message, just send it and then return.
                    return;
                },
                _ => (),
            }
            // Send the message
            match sender.send_message(&message) {
                Ok(()) => {
                    println!("Send Loop: Sent: {:?}", &message);
                },
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message: Message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(Message::close());
                    return;
                }
            };
            match message.opcode {
                Type::Close => {
                    // Got a close message, so send a close message and return
                    let _ = tx_1.send(Message::close());
                    return;
                }
                Type::Ping => match tx_1.send(Message::pong(message.payload)) {
                    // Send a pong in response
                    Ok(()) => (),
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        return;
                    }
                },
                // Say what we received
                _ => {
                    println!("Receive Loop:");
                    println!("{}", from_utf8(&*message.payload).unwrap());
                }
            }
        }
    });

    tx.send(Message::text("2probe"));
    tx.send(Message::text("5"));

    loop {
        let mut input = String::new();

        stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();

        let message = match trimmed {
            "/close" => {
                // Close the connection
                let _ = tx.send(Message::close());
                break;
            }
            // Send a ping
            "/ping" => Message::ping(b"PING".to_vec()),
            // Otherwise, just send text
            _ => Message::text(trimmed.to_string()),
        };

        match tx.send(message) {
            Ok(()) => (),
            Err(e) => {
                println!("Main Loop: {:?}", e);
                break;
            }
        }
    }

    // We're exiting

    println!("Waiting for child threads to exit");

    let _ = send_loop.join();
    let _ = receive_loop.join();

    println!("Exited");
}
