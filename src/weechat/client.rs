use std::io::{stdin, stdout};
use std::sync::mpsc::channel;
use std::thread;

use websocket::client::ClientBuilder;
use websocket::result::WebSocketError;
use websocket::{Message, OwnedMessage};

use nom::IResult;
use weechat::parser;

const CONNECTION: &'static str = "ws://127.0.0.1:8100/weechat";

pub fn connect() {
    println!("Connecting to {}", CONNECTION);

    // Create a client object to connect to the remote.
    let mut client = ClientBuilder::new(CONNECTION)
        .unwrap()
        .add_protocol("weechat")
        .connect_insecure()
        .unwrap();

    let _ = client.set_nodelay(true).unwrap();
    // let _ = client.set_nonblocking(true).unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

    let _ = match sender.send_message(&OwnedMessage::Text(
        "init compression=off,password=ab\n".to_string(),
    )) {
        Ok(()) => (println!("Connected.")),
        Err(err) => println!("Initialization Error: {:?}", err),
    };

    let _ = match sender.send_message(&OwnedMessage::Text("(1) info version\n".to_string())) {
        Ok(()) => (println!("Requesting version.")),
        Err(err) => println!("Initialization Error: {:?}", err),
    };

    // Setup a loop to recv messages from remote.
    receiver.recv_dataframe().unwrap();

    let _ = match sender.send_message(&OwnedMessage::Text("sync\n".to_string())) {
        Ok(()) => (println!("Sent sync...")),
        Err(err) => println!("Couldn't send ping: {}", err),
    };

    // Setup a loop to recv messages from remote.
    let recv_loop = thread::spawn(move || {
        for message in receiver.incoming_dataframes() {
            let message_data = message.unwrap().data;

            let result = match parser::parse_header(&message_data) {
                Ok(res) => res,
                Err(err) => panic!("Error parsing header: {:?}", err),
            };
            let weechat_header = result.1;
            println!("Total length of response: {:?}", weechat_header.body_length);
            println!("Response is compressed: {:?}", weechat_header.compressed);

            let result = match parser::parse_identifier(result.0) {
                Ok(res) => res,
                Err(err) => panic!("Error parsing identifier: {:?}", err),
            };

            let weechat_id = result.1;
            println!("The identifier was: {:?}", weechat_id);

            let result = match parser::parse_type(result.0) {
                Ok(res) => res,
                Err(err) => panic!("Error parsing obj type: {:?}", err),
            };

            let objtype = result.1;
            println!("Found obj type: {:?}", objtype);

            match parser::parse_hdata(result.0) {
                Ok(res) => println!("The hdata object was: {:?}", res.1),
                Err(err) => println!("Error parsing hdata: {:?}", err),
            };
        }
    });

    let _ = recv_loop.join();

    // let (tx, rx) = channel();
    // let tx_1 = tx.clone();

    // let send_loop = thread::spawn(move || {
    //     loop {
    //         // Send loop
    //         let message = match rx.recv() {
    //             Ok(m) => m,
    //             Err(err) => {
    //                 println!("Send Loop: {:?}", err);
    //                 return;
    //             },
    //         }
    //         match message {
    //             OwnedMessage::Close(_) => {
    //                 let _ = sender.send_message(&message);
    //                 return;
    //             }
    //             _ => (),
    //         }
    //         println!("Sending message: {:?}", message);
    //         match sender.send_message(&message) {
    //             Ok(()) => (),
    //             Err(err) => {
    //                 println!("Send Loop: {:?}", err);
    //                 let _ = sender.send_message(&Message::close());
    //                 return;
    //             }
    //         }
    //     }
    // });
}
