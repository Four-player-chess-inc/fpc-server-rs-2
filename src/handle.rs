use crate::to_message::ToMessage;
use crate::tungstenite::Error;
use crate::GameFabric;
use fpc_proto::from_client::FromClient;
use fpc_proto::from_client::MatchmakingQueue::Register;
use fpc_proto::to_client::UnspecifiedError;
use futures::stream::{BoxStream, SelectAll};
use futures::{SinkExt, Stream, StreamExt};
use log::{debug, error};
use matchmaker::inqueue::InqueueSender;
use matchmaker::{Event, Matchmaker};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

enum State {
    Idle,
    Inqueue(InqueueSender),
    Ingame,
}

impl State {
    fn is_idle(&self) -> bool {
        match self {
            Self::Idle => true,
            _ => false,
        }
    }

    fn to_inqueue(&mut self, iq: InqueueSender) {
        *self = State::Inqueue(iq);
    }
}

#[derive(Debug)]
enum StreamConcat {
    Net(Result<tungstenite::Message, tungstenite::Error>),
    Matchmaker(Event),
    Game,
}

fn net_process(net: Result<Message, Error>) {
    //let unwrap = match net
}

pub(crate) async fn handle(
    addr: SocketAddr,
    tcp_stream: TcpStream,
    matchmaker: Arc<Mutex<Matchmaker>>,
    game_fabric: Arc<Mutex<GameFabric>>,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await.unwrap();
    let (mut net_tx, mut net_rx) = ws_stream.split();

    let mut net_rx_map = net_rx.map(|i| StreamConcat::Net(i));

    let mut select: SelectAll<BoxStream<StreamConcat>> = SelectAll::new();

    select.push(net_rx_map.boxed());

    let mut state = State::Idle;

    let mut p_name = None;

    //let mut raw_err = RawErr(addr, &mut net_tx);

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
                            FromClient::MatchmakingQueue(Register { name }) => if matches!(state, State::Idle) {
                                p_name = Some(name);
                            }

                        },
                        Err(e) => {
                            //raw_err.execute("qwe","qwe");
                            let desc = "Error while parsing msg";
                            error!("{:?}, {}, msg: {:?}", addr, desc, e);
                            if let Ok(m) = (UnspecifiedError { desc: desc.into() }).try_to_message()
                            {
                                net_tx.send(m).await;
                            }
                        }
                    }
                    /*if m == "queue".to_string() && state.is_idle() {
                        if let Ok(inqueue) = matchmaker.lock().await.join().await {
                            let (inqueue_tx, inqueue_rx) = inqueue.split();
                            let inqueue_rx_map = inqueue_rx.map(|i| StreamConcat::Matchmaker(i));
                            select.push(inqueue_rx_map.boxed());
                            state.to_inqueue(inqueue_tx);
                        }
                    }*/
                }
                Err(Error::ConnectionClosed) | Ok(Message::Close(_)) => (),
                _ => {
                    let desc = "Error while recv()";
                    error!("{:?}, {}, msg: {:?}", addr, desc, net);
                    if let Ok(m) = (UnspecifiedError { desc: desc.into() }).try_to_message() {
                        net_tx.send(m).await;
                    }
                }
            },
            StreamConcat::Matchmaker(_) => (),
            StreamConcat::Game => (),
        }
    }
}
