use crate::process::MmRegisterErr;
use fpc_proto::to_client::{MatchmakingQueue, Register, ToClient, UnspecifiedError};
use tokio_tungstenite::tungstenite::Message;

pub trait TryToMessage {
    fn try_to_msg(&self) -> serde_json::Result<Message>;
}

impl TryToMessage for UnspecifiedError {
    fn try_to_msg(&self) -> serde_json::Result<Message> {
        let json = serde_json::to_string(self)?;
        Ok(Message::Text(json))
    }
}

impl TryToMessage for ToClient {
    fn try_to_msg(&self) -> serde_json::Result<Message> {
        let json = serde_json::to_string(self)?;
        Ok(Message::Text(json))
    }
}

pub(crate) struct ToClientWrap {
    pub(crate) to_client: ToClient,
}
