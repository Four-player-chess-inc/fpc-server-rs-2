use serde_json::Result;
use tokio_tungstenite::tungstenite::Message;

pub trait ToMessage {
    fn try_to_message(&self) -> Result<Message>;
}

impl ToMessage for fpc_proto::to_client::UnspecifiedError {
    fn try_to_message(&self) -> Result<Message> {
        let json = serde_json::to_string(self)?;
        Ok(Message::Text(json))
    }
}
