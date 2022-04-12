use websocket::client::{
    ClientBuilder,
    sync::Client
};
use websocket::header::{
    Headers,
    WebSocketVersion,
    extensions::Extension
};
use websocket::{OwnedMessage, Message};
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::thread;

const SOCKET_URL: &'static str = "wss://gateway.discord.gg/?encoding=json&v=9";

fn generate_socket_key() -> [u8; 16] {
    // Random base64 encoded bytes
    let mut bytes = [0u8; 16];
    
    for i in 0..16 {
        bytes[i] = rand::random::<u8>();
    }

    bytes
}

fn get_headers() -> Headers {
    let mut headers = Headers::new();
    headers.set_raw("User-Agent", vec!(b"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Safari/605.1.15".to_vec()));
    headers.set_raw("Sec-WebSocket-Version", vec!(b"13".to_vec()));
    headers.set_raw("Sec-WebSocket-Extensions", vec!(b"permessage-deflate, client_max_window_bits".to_vec()));
    headers.set_raw("Cache-Control", vec!(b"no-cache".to_vec()));
    headers.set_raw("Connection", vec!(b"Upgrade".to_vec()));
    headers.set_raw("Pragma", vec!(b"no-cache".to_vec()));
    headers.set_raw("Host", vec!(b"gateway.discord.gg".to_vec()));
    headers
}

pub fn create_connection() {
    let mut client = ClientBuilder::new(SOCKET_URL)
        .unwrap()
        .add_extension(Extension { name: "permessage-defalte".to_string(), params: vec![] })
        .add_extension(Extension { name: "client_max_window_bits".to_string(), params: vec![] })
        .origin("https://discord.com".to_string())
        .key(generate_socket_key())
        .version(WebSocketVersion::WebSocket13)
        .add_protocol("websocket")
        .custom_headers(&get_headers())
        .connect_secure(None)
        .map_err(|e| eprintln!("Error connecting: {}", e))
        .unwrap();
        
    
    loop {
        let message = match client.recv_message() {
            Ok(msg) => msg,
            Err(e) => {
                println!("{:?}", e);
                if let Err(e) = client.shutdown() {
                    println!("{:?}", e);
                }
                break;
            }
        };

        println!("{:?}", message);

        match message {
            OwnedMessage::Close(_) => {
                println!("Received close message");

                if let Err(e) = client.shutdown() {
                    println!("Error shutting down client: {:?}", e);
                }
                break;
            }
            OwnedMessage::Ping(data) => {
                println!("Received ping message");
                client.send_message(&OwnedMessage::Pong(data)).unwrap();
            }
            OwnedMessage::Pong(_) => {
                println!("Received pong message");
            }
            OwnedMessage::Text(text) => {
                println!("Received text message: {}", text);
            }
            OwnedMessage::Binary(data) => {
                println!("Received binary message: {:?}", data);
            }
        }
    }
}