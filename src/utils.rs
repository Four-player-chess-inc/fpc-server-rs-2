use crate::to_message::ToMessage;
use fpc_proto::to_client::UnspecifiedError;
use futures::stream::SplitSink;
use futures::SinkExt;
use log::error;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

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
