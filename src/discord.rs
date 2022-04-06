use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;


struct Handler;

#[async_trait]
impl EventHandler for Handler {}

pub async fn create_client(token: String) -> Client {
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    
    client
}
