use std::io::{stdin, stdout};
use std::sync::mpsc::channel;
use std::thread;

use websocket::result::WebSocketError;
use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

use byte::*;
use byte::ctx::*;

const CONNECTION: &'static str = "ws://127.0.0.1:8100/weechat";

// fn read_weechat_response(stream: &TcpStrem) {
//     let mut response_len = [0; 4];

//     // We first read the length of the response.
//     stream.read_exact(&mut response_len);
//     println!("Response of length: {:?}", response_len);
// }


// fn send_one_command(command: &str, mut stream: &TcpStream) {
//     let mut command_bytes = ASCII.encode(&command, EncoderTrap::Strict).unwrap();
//     command_bytes.push('\n' as u8);
//     command_bytes.push('\r' as u8);

//     println!("Sending command: {}", command);
//     stream.write_all(&command_bytes).expect("Failed to send command.");

// }


// fn init_weechat(mut stream: &TcpStream) {

//     stream.write_all(b"init\n\r").expect("Failed to initialize.");

//     println!("Initialized Weechat!");
// }


// pub fn parse_weechat_response(message: &OwnedMessage) {
// }

// enum WechatTypes {
//     Char,
//     Int,
//     Long,
//     String,
//     Buffer,
//     Pointer,
//     Time,
//     HashTable,
//     Hdata,
//     Info,
//     InfoList,
// }


struct WeeChatMessage<'a> {
    body_length: u32,
    compressed: bool,
    identifier: &'a str,
}


impl<'a> TryRead<'a, Endian> for WeeChatMessage<'a> {

    fn try_read(bytes: &'a [u8], endian: Endian) -> Result<(Self, usize)>  {
        let offset = &mut 0;
        let body_len = bytes.read_with::<u32>(offset, endian)?;
        let compressed = bytes.read::<bool>(offset)?;

        let id_length = bytes.read_with::<u32>(offset, endian)? as usize;
        let id = bytes.read_with::<&str>(offset, Str::Len(id_length))?;

        let message = WeeChatMessage {
            body_length: body_len,
            compressed: compressed,
            identifier: id,
        };

        Ok((message, *offset))
    }
}


struct WeeChatHeader {
    body_length: u32,
    compressed: bool,
}

impl<'a> TryRead<'a, Endian> for WeeChatHeader {

    fn try_read(bytes: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
        let offset = &mut 0;

        let header = WeeChatHeader {
            body_length: bytes.read_with::<u32>(offset, BE)?,
            compressed: bytes.read::<bool>(offset)?,
        };
        Ok((header, *offset))
    }
}


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


    let _ = match sender.send_message(&OwnedMessage::Text("init compression=off,password=ab\n".to_string())) {
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
            // println!("Got response: {:?}", message.unwrap());
            let message_data = message.unwrap().data;
            let weemessage: WeeChatMessage =  message_data.read_with(&mut 0, BE).unwrap();
            println!("Total length of response: {:?}", weemessage.body_length);
            println!("Response is compressed: {:?}", weemessage.compressed);
            println!("Message identifier: {:?}", weemessage.identifier);
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

    // let receive_loop = thread::spawn(move || {
    //     // Receive loop
    //     for message in receiver.incoming_messages() {

    //         println!("Got a reponse on websocket: {:?}", message);

    //         let message = match message {
    //             Ok(m) => m,
    //             Err(WebSocketError::NoDataAvailable) => return,
    //             Err(err) => {
    //                 eprintln!("Receive Loop: {:?}", err);
    //                 let _ = tx_1.send(OwnedMessage::Close(None));
    //                 return;
    //             }
    //         };

    //         match message {
    //             OwnedMessage::Close(_) => {
    //                 // Close the connection.
    //                 println!("recv Close");
    //                 let _ = tx_1.send(OwnedMessage::Close(None));
    //                 return;
    //             }
    //             OwnedMessage::Ping(data) => {
    //                 println!("recv Ping");
    //                 match tx_1.send(OwnedMessage::Pong(data)) {
    //                     Ok(()) => println!("send Pong"),
    //                     Err(e) => {
    //                         println!("Receive Loop: {:?}", e);
    //                         return;
    //                     }
    //                 }
    //             }
    //             OwnedMessage::Binary(msg) => println!("Receive Loop: {:?}", msg),
    //             _ => println!("Unknown response type"),
    //         };
    //     }
    // });


    // loop {
    //     let mut  input = String::new();
    //     println!("Enter command:");
    //     stdin().read_line(&mut input).unwrap();
    //     input.push_str(&"\n".to_string());

    //     let trimmed = input.trim();
    //     let message = match trimmed {
    //         "/close" => {
    //             let _ = tx.send(OwnedMessage::Close(None));
    //             break;
    //         }
    //         "/ping" => OwnedMessage::Ping(b"PING".to_vec()),
    //         // Othewise just send the bare text.
    //         _ => OwnedMessage::Text(trimmed.to_string()),
    //     };

    //     match tx.send(message) {
    //         Ok(()) => (),
    //         Err(err) => {
    //             println!("Main Loop: {:?}", err);
    //             break;
    //         }
    //     }

    // }

    // // We're exiting

    // println!("Waiting for child threads to exit");
    // let _ = send_loop.join();
    // let _ = receive_loop.join();
    // println!("Exited");
}
