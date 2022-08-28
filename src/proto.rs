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

pub(crate) fn unspec_err(desc: &str) -> UnspecifiedError {
    UnspecifiedError { desc: desc.into() }
}

/*pub(crate) fn reg_ok() -> ToClient {
    ToClient::MatchmakingQueue(MatchmakingQueue::Register(Register::Ok {}))
}*/

/*#[macro_export]
macro_rules! reg_ok {
    () => {
        crate::to_client::ToClient::MatchmakingQueue(crate::to_client::MatchmakingQueue::Register(crate::to_client::Register::Ok {}))
    }
}*/

pub(crate) struct ToClientWrap {
    pub(crate) to_client: ToClient,
}
