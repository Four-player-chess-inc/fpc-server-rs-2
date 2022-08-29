/*pub(crate) struct RawErr<'a>(
    pub(crate) SocketAddr,
    pub(crate) &'a mut SplitSink<WebSocketStream<TcpStream>, Message>,
);

impl<'a> RawErr<'a> {
    pub(crate) async fn execute<T: Debug>(&mut self, desc: &str, e: T) {
        error!("{:?}, {}, msg: {:?}", self.0, desc, e);
        if let Ok(m) = (UnspecifiedError { desc: desc.into() }).try_to_message() {
            self.1.send(m).await;
        }
    }
}*/

//pub trait try_send

use async_trait::async_trait;
use futures::stream::SplitSink;
use futures::SinkExt;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[async_trait]
pub trait SendIfMsg {
    async fn send_if_enc<K: Serialize + Send>(&mut self, t: K);
}

#[async_trait]
impl SendIfMsg for SplitSink<WebSocketStream<TcpStream>, Message> {
    async fn send_if_enc<K: Serialize + Send>(&mut self, t: K) {
        let x = serde_json::to_string(&t);
        if let Ok(j) = x {
            #[allow(unused_must_use)]
            {
                self.send(Message::Text(j)).await;
            }
        }
    }
}
