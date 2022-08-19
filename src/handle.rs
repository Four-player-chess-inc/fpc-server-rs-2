use crate::tungstenite::Error;
use crate::GameFabric;
use futures::stream::{BoxStream, SelectAll};
use futures::{Stream, StreamExt};
use log::{debug, error};
use matchmaker::inqueue::InqueueSender;
use matchmaker::{Event, Matchmaker};
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
enum NetMM {
    Net(Result<tungstenite::Message, tungstenite::Error>),
    Event(Event),
}

fn net_process(net: Result<Message, Error>) {
    //let unwrap = match net
}

pub(crate) async fn handle(
    tcp_stream: TcpStream,
    matchmaker: Arc<Mutex<Matchmaker>>,
    game_fabric: Arc<Mutex<GameFabric>>,
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await.unwrap();
    let (net_tx, mut net_rx) = ws_stream.split();

    let mut net_rx_map = net_rx.map(|i| NetMM::Net(i));

    let mut select: SelectAll<BoxStream<NetMM>> = SelectAll::new();

    select.push(net_rx_map.boxed());

    let mut state = State::Idle;

    while let Some(pdu) = select.next().await {
        debug!("msg: {:?}, select count: {}", &pdu, select.len());
        match pdu {
            NetMM::Net(net) => match net {
                Ok(m) => {
                    if m == Message::text("queue".to_string()) && state.is_idle() {
                        if let Ok(inqueue) = matchmaker.lock().await.join().await {
                            let (inqueue_tx, inqueue_rx) = inqueue.split();
                            let inqueue_rx_map = inqueue_rx.map(|i| NetMM::Event(i));
                            select.push(inqueue_rx_map.boxed());
                            state.to_inqueue(inqueue_tx);
                        }
                    }
                }
                Err(e) => error!("{}", e),
            },
            NetMM::Event(_) => (),
        }
    }
}
