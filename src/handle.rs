use crate::process::mm_register;
use crate::proto::{ToClientWrap, TryToMessage};
use crate::tungstenite::Error;
use crate::utils::SendIfMsg;
use crate::{proto, GameFabric};
use fpc_proto::from_client::FromClient;
use fpc_proto::from_client::MatchmakingQueue::{HeartbeatCheck, Leave, Register};
use fpc_proto::to_client::MatchmakingQueue;
use fpc_proto::to_client::ToClient;
use fpc_proto::{mm_continue, mm_hb_check, mm_player_kick, mm_reg_ok, unspec_err};
use futures::stream::{BoxStream, SelectAll};
use futures::{SinkExt, StreamExt};
use log::{debug, error};
use matchmaker::inqueue::InqueueSender;
use matchmaker::{Event, Matchmaker};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

pub(crate) enum State {
    Idle,
    Inqueue(InqueueSender),
    Ingame,
}

impl Default for State {
    fn default() -> Self {
        State::Idle
    }
}

impl State {
    pub(crate) fn is_idle(&self) -> bool {
        match self {
            Self::Idle => true,
            _ => false,
        }
    }

    pub(crate) fn to_inqueue(&mut self, iq: InqueueSender) {
        *self = State::Inqueue(iq);
    }

    pub(crate) fn to_idle(&mut self) {
        *self = State::Idle;
    }
}

#[derive(Debug)]
pub(crate) enum StreamConcat {
    Net(Result<tungstenite::Message, tungstenite::Error>),
    Matchmaker(Event),
    Game,
}

fn net_process(net: Result<Message, Error>) {
    //let unwrap = match net
}

#[derive(Default)]
pub(crate) struct Storage {
    pub(crate) name: Option<String>,
    pub(crate) state: State,
}

pub(crate) async fn handle(
    addr: SocketAddr,
    tcp_stream: TcpStream,
    matchmaker: Arc<Mutex<Matchmaker>>,
    game_fabric: Arc<Mutex<GameFabric>>,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await.unwrap();
    let (mut net_tx, net_rx) = ws_stream.split();

    let net_rx_map = net_rx.map(|i| StreamConcat::Net(i));
    let mut select: SelectAll<BoxStream<StreamConcat>> = SelectAll::new();
    select.push(net_rx_map.boxed());

    let mut storage = Storage::default();

    while let Some(pdu) = select.next().await {
        debug!(
            "{:?}, msg: {:?}, select count: {}",
            addr,
            &pdu,
            select.len()
        );
        match pdu {
            StreamConcat::Net(net) => match net {
                Ok(Message::Text(raw)) => {
                    match serde_json::from_str::<FromClient>(&raw) {
                        Ok(proto_msg) => match proto_msg {
                            FromClient::MatchmakingQueue(Register { name }) => {
                                // TODO: log error if mm queque full
                                // TODO: validate name
                                let to_client_wrap: ToClientWrap = mm_register(
                                    &mut storage,
                                    matchmaker.clone(),
                                    &mut select,
                                    name,
                                )
                                .await
                                .into();
                                net_tx.send_if_enc(to_client_wrap.to_client).await;
                            }
                            FromClient::MatchmakingQueue(Leave {}) => {
                                if let State::Inqueue(ref inq) = storage.state {
                                    inq.leave();
                                }
                            }
                            FromClient::MatchmakingQueue(HeartbeatCheck {}) => {
                                if let State::Inqueue(ref inq) = storage.state {
                                    inq.confirm();
                                }
                            }
                        },
                        Err(e) => {
                            let desc = "Error while parsing msg";
                            error!("{:?}, {}, msg: {:?}", addr, desc, e);
                            net_tx.send_if_enc(unspec_err!(desc)).await;
                        }
                    }
                }
                Err(Error::ConnectionClosed) | Ok(Message::Close(_)) => (),
                _ => {
                    let desc = "Error while recv()";
                    error!("{:?}, {}, msg: {:?}", addr, desc, net);
                    net_tx.send_if_enc(unspec_err!(desc)).await;
                }
            },
            StreamConcat::Matchmaker(ev) => match ev {
                Event::Kicked { reason } => {
                    storage.state.to_idle();
                    net_tx
                        .send_if_enc(mm_player_kick!(format! {"{:?}", reason}))
                        .await;
                }
                Event::ConfirmRequired { timeout } => {
                    net_tx.send_if_enc(mm_hb_check!(timeout.as_secs())).await;
                }
                Event::WaitForQuorum => net_tx.send_if_enc(mm_continue!()).await,
                Event::Ready => todo!(),
            },
            StreamConcat::Game => (),
        }
    }
}
