mod game_fabric;
mod handle;
mod process;
mod proto;
mod utils;

use crate::game_fabric::GameFabric;
use crate::handle::handle;
use env_logger::Builder;
use log::LevelFilter;
use matchmaker::Matchmaker;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite;

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
