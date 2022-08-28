use crate::handle::{State, Storage, StreamConcat};
use crate::proto::ToClientWrap;
use fpc_proto::{reg_err_already, reg_err_bad_name, reg_err_unspec, reg_ok};
use futures::stream::{BoxStream, SelectAll};
use futures::StreamExt;
use matchmaker::Matchmaker;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub(crate) enum MmRegisterErr {
    #[error("State must be Idle to join matchmaking queue")]
    StateNotIdle,
    #[error("Error while join matchmaking queue")]
    MmJoin(#[from] matchmaker::JoinErr),
}

pub(crate) async fn mm_register(
    storage: &mut Storage,
    matchmaker: Arc<Mutex<Matchmaker>>,
    select: &mut SelectAll<BoxStream<'_, StreamConcat>>,
    name: String,
) -> Result<(), MmRegisterErr> {
    if !matches!(storage.state, State::Idle) {
        return Err(MmRegisterErr::StateNotIdle);
    }
    let inqueue = matchmaker.lock().await.join().await?;
    let (inqueue_tx, inqueue_rx) = inqueue.split();
    let inqueue_rx_map = inqueue_rx.map(|i| StreamConcat::Matchmaker(i));
    select.push(inqueue_rx_map.boxed());
    storage.state.to_inqueue(inqueue_tx);
    storage.name = Some(name);
    Ok(())
}

impl From<Result<(), MmRegisterErr>> for ToClientWrap {
    fn from(r: Result<(), MmRegisterErr>) -> Self {
        let to_client = match r {
            Ok(()) => reg_ok!(),
            Err(MmRegisterErr::StateNotIdle) => {
                reg_err_already!("already in mm queue or in game")
            }
            Err(MmRegisterErr::MmJoin(matchmaker::JoinErr::DuplicateUniqueId)) => {
                reg_err_unspec!("queue full")
            }
        };
        ToClientWrap { to_client }
    }
}
