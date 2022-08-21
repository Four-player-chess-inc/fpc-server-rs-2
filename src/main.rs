mod game_fabric;
mod handle;
mod to_message;
mod utils;

use crate::game_fabric::GameFabric;
use crate::handle::handle;
use env_logger::Builder;
use futures::stream::{select, select_all, SelectAll, SplitStream};
use futures::{SinkExt, Stream, StreamExt};
use log::{debug, error, info, LevelFilter};
use matchmaker::inqueue::{InqueueReceiver, InqueueSender};
use matchmaker::{Event, Matchmaker};
use std::pin::Pin;
use std::{collections::HashMap, env, io::Error as IoError, net::SocketAddr, sync::Arc};
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{tungstenite, WebSocketStream};

#[tokio::main]
async fn main() {
    Builder::new().filter(None, LevelFilter::Debug).init();

    let try_socket = TcpListener::bind("0.0.0.0:32145").await;
    let listener = try_socket.expect("Failed to bind");

    let mm = Arc::new(Mutex::new(Matchmaker::new()));
    let gf = Arc::new(Mutex::new(GameFabric::new()));

    while let Ok((tcp_stream, addr)) = listener.accept().await {
        tokio::spawn(handle(addr, tcp_stream, mm.clone(), gf.clone()));
    }
}
