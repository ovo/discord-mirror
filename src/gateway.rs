use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};
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
    d: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Heartbeat {
    heartbeat_interval: i64,

    #[serde(rename = "_trace")]
    trace: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageEvent {
    #[serde(rename = "type")]
    d_type: i64,
    tts: bool,
    timestamp: String,
    referenced_message: Option<serde_json::Value>,
    pinned: bool,
    nonce: String,
    mentions: Vec<Option<serde_json::Value>>,
    mention_roles: Vec<Option<serde_json::Value>>,
    mention_everyone: bool,
    member: Member,
    id: String,
    flags: i64,
    embeds: Vec<Option<serde_json::Value>>,
    edited_timestamp: Option<serde_json::Value>,
    content: String,
    components: Vec<Option<serde_json::Value>>,
    channel_id: String,
    author: Author,
    attachments: Vec<Option<serde_json::Value>>,
    guild_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    username: String,
    public_flags: i64,
    id: String,
    discriminator: String,
    avatar: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Member {
    roles: Vec<Option<serde_json::Value>>,
    mute: bool,
    joined_at: String,
    hoisted_role: Option<serde_json::Value>,
    flags: i64,
    deaf: bool,
}

pub struct Gateway {
    token: String,
    server_id: String,
    heartbeat_time: Instant,
    client: Client<TlsStream<TcpStream>>,
    heartbeat_interval: i32,
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

fn generate_socket_key() -> [u8; 16] {
    let mut bytes = [0u8; 16];

    for i in 0..16 {
        bytes[i] = rand::random::<u8>();
    }

    bytes
}

fn new_client() -> Result<Client<TlsStream<TcpStream>>, WebSocketError> {
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

impl Gateway {
    pub fn new(token: String, server_id: String) -> Self {
        let client = new_client()
            .map_err(|e| println!("Could not connect to socket: {}", e))
            .unwrap();

        Self {
            token,
            server_id,
            heartbeat_time: Instant::now(),
            client,
            heartbeat_interval: 41250,
        }
    }

    fn authenticate(&mut self) {
        let token = &self.token;
        let client = &mut self.client;

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

        client
            .send_message(&Message::text(msg.to_string()))
            .map_err(|e| eprintln!("Error authenticating: {:?}", e))
            .unwrap();
    }

    pub fn listen(&mut self) {
        loop {
            let client = &mut self.client;
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

            self.message_event(message);

        }
    }

    fn message_event(&mut self, message: OwnedMessage) {
        let client = &mut self.client;

        match message {
            OwnedMessage::Close(_) => {
                println!("Received close message - resetting connection");
                return;
            }
            OwnedMessage::Ping(data) => {
                println!("Received ping message");
                client
                    .send_message(&OwnedMessage::Pong(data.to_vec()))
                    .unwrap();
            }
            OwnedMessage::Pong(_) => {
                println!("Received pong message");
            }
            OwnedMessage::Text(text) => {
                let deserialized_msg: Payload = serde_json::from_str(&text).unwrap();

                match deserialized_msg.op {
                    10 => {
                        // Authenticate and set heartbeat interval;
                        self.authenticate();
                    }
                    0 => {
                        /*
                        match deserialized_msg.t {
                            "MESSSAGE_CREATE" =>
                        }
                        */
                        // Check if t is MESSAGE_CREATE:
                        if let Some(t) = deserialized_msg.t {
                            if t == "MESSAGE_CREATE" {
                                let message_event: MessageEvent =
                                    serde_json::from_value(deserialized_msg.d).unwrap();
                                println!(
                                    "{:?}#{:?} in {:?}: {:?}",
                                    message_event.author.username,
                                    message_event.author.discriminator,
                                    message_event.channel_id,
                                    message_event.content
                                );
                            }
                        }
                    }
                    _ => println!("{}\n{:?}\n", deserialized_msg.op, deserialized_msg),
                }
            }
            OwnedMessage::Binary(data) => {
                println!("Received binary message: {:?}", data);
            }
        }
    }
}

fn heartbeat(client: &mut Client<TlsStream<TcpStream>>, count: i32) {
    loop {
        let msg = json!({
            "op": 1,
            "d": count,
        });
        client
            .send_message(&OwnedMessage::Text(msg.to_string()))
            .unwrap();
        thread::sleep(Duration::from_millis(count as u64));
    }
}
