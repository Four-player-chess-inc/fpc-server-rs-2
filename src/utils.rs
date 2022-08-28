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

//pub trait try_send
