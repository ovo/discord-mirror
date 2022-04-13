use serde::{Serialize, Deserialize};
use serde_json::json;
use std::net::TcpStream;
use websocket::client::{sync::Client, ClientBuilder};
use websocket::header::{extensions::Extension, Headers, WebSocketVersion};
use websocket::native_tls::TlsStream;
use websocket::{Message, OwnedMessage, WebSocketError};

const SOCKET_URL: &'static str = "wss://gateway.discord.gg/?encoding=json&v=9";
const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Safari/605.1.15";


#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    op: i32,
    s: Option<i32>,
    t: Option<String>,
    d: serde_json::Value
}


fn generate_socket_key() -> [u8; 16] {
    let mut bytes = [0u8; 16];

    for i in 0..16 {
        bytes[i] = rand::random::<u8>();
    }

    bytes
}

fn get_headers() -> Headers {
    let mut headers = Headers::new();
    headers.set_raw("User-Agent", vec![USER_AGENT.as_bytes().to_vec()]);
    headers.set_raw("Sec-WebSocket-Version", vec![b"13".to_vec()]);
    headers.set_raw(
        "Sec-WebSocket-Extensions",
        vec![b"permessage-deflate, client_max_window_bits".to_vec()],
    );
    headers.set_raw("Cache-Control", vec![b"no-cache".to_vec()]);
    headers.set_raw("Connection", vec![b"Upgrade".to_vec()]);
    headers.set_raw("Pragma", vec![b"no-cache".to_vec()]);
    headers.set_raw("Host", vec![b"gateway.discord.gg".to_vec()]);
    headers
}

pub fn create_client() -> Result<Client<TlsStream<TcpStream>>, WebSocketError> {
    ClientBuilder::new(SOCKET_URL)
        .unwrap()
        .add_extension(Extension {
            name: "permessage-defalte".to_string(),
            params: vec![],
        })
        .add_extension(Extension {
            name: "client_max_window_bits".to_string(),
            params: vec![],
        })
        .origin("https://discord.com".to_string())
        .key(generate_socket_key())
        .version(WebSocketVersion::WebSocket13)
        .add_protocol("websocket")
        .custom_headers(&get_headers())
        .connect_secure(None)
}

pub fn listen(mut client: Client<TlsStream<TcpStream>>) {
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
                println!("Received close message - resetting connection");
                let new_client = create_client().unwrap();
                listen(new_client);
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
                let deserialized_msg: Payload = serde_json::from_str(&text).unwrap();

                match deserialized_msg.op {
                    10 => authenticate(&mut client, std::env::var("USER_DISCORD_TOKEN").expect("Expected user token in env")),
                    _ => ()
                }
                println!("{}", deserialized_msg.op);
                
            }
            OwnedMessage::Binary(data) => {
                println!("Received binary message: {:?}", data);
            }
        }
    }
}

fn authenticate(client: &mut Client<TlsStream<TcpStream>>, token: String) {
    let msg = json!({
      "op": 2,
      "d": {
        "token": token,
        "capabilities": 253,
        "properties": {
          "os": "Mac OS X",
          "browser": "Safari",
          "device": "",
          "system_locale": "en-US",
          "browser_user_agent": USER_AGENT,
          "browser_version": "15.4",
          "os_version": "10.15.7",
          "referrer": "https://www.google.com/",
          "referring_domain": "www.google.com",
          "search_engine": "google",
          "referrer_current": "",
          "referring_domain_current": "",
          "release_channel": "stable",
          "client_build_number": 123680,
          "client_event_source": null
        },
        "presence": {
          "status": "online",
          "since": 0,
          "activities": [],
          "afk": false
        },
        "compress": false,
        "client_state": {
          "guild_hashes": {},
          "highest_last_message_id": "0",
          "read_state_version": 0,
          "user_guild_settings_version": -1,
          "user_settings_version": -1
        }
      }
    });

    client.send_message(&Message::text(msg.to_string()))
        .map_err(|e| eprintln!("Error authenticating: {:?}", e))
        .unwrap();
}
