use crate::log::*;
use flume::{unbounded, Receiver, Sender};

/// Abstraction for an "envelope" pattern, in which we send some type `T` to another thread and expect to get a `U` back.
/// An envelope is a pair of some request type `T` and a sender for the `U` reply to it, like the return address on a physical mail envelope.
///
/// `T` and `U` must be `Send`.
pub type Envelope<T, U> = (T, Sender<U>);
/// Shorthand for generating a pair of `Envelope` channels.
pub fn envelope_pair<U: Send, T: Send>() -> (Sender<Envelope<T, U>>, Receiver<Envelope<T, U>>) {
    unbounded()
}
#[derive(Clone)]
/// A struct abstracting over "server" threads which react to incoming events.
pub struct ChannelServer<T: Send, U: Send, V: HandleRequest<T, U>> {
    request_rx: Receiver<Envelope<T, U>>,
    handler: V,
}
impl<T: Send, U: Send, V: HandleRequest<T, U>> ChannelServer<T, U, V> {
    pub fn handle(&mut self) {
        // TODO: handle timeouts
        let (req, reply_tx) = match self.request_rx.recv() {
            Ok(t) => {
                trace!("A ChannelServer got a request!");
                t
            }
            Err(_) => {
                panic!("A ChannelServer could not receive a request!");
            }
        };
        let reply = self.handler.handle(req);
        match reply_tx.send(reply) {
            Ok(_) => {
                trace!("A ChannelServer replied to a request!");
            }
            Err(_) => {
                panic!("A ChannelServer could not respond to a request!");
            }
        }
    }
}
pub trait HandleRequest<T: Send, U: Send> {
    /// React to an incoming message.
    fn handle(&mut self, msg: T) -> U;
}
#[derive(Clone)]
pub struct ChannelClient<T: Send, U: Send> {
    /// For cloning when sending a request.
    reply_tx: Sender<U>,
    reply_rx: Receiver<U>,
    request_tx: Sender<Envelope<T, U>>,
}
impl<T: Send, U: Send> ChannelClient<T, U> {
    /// Make a request and return the appropriate response based on the handler's implementation.
    pub fn request(&mut self, msg: T) -> U {
        let msg = (msg, self.reply_tx.clone());
        match self.request_tx.send(msg) {
            Ok(_) => (),
            Err(_) => {
                let text = "A ChannelClient could not make a request!";
                error!("{}", text);
                panic!(text);
            }
        };
        match self.reply_rx.recv() {
            Ok(u) => u,
            Err(_) => {
                let text = "A ChannelClient could not receive a reply!";
                error!("{}", text);
                panic!(text)
            }
        }
    }
}
/// Returns a client-server pair as a tuple, analoguously to MPSC channel creation.
pub fn client_server<T, U, V>(handler: V) -> (ChannelClient<T, U>, ChannelServer<T, U, V>)
where
    T: Send,
    U: Send,
    V: HandleRequest<T, U>,
{
    let (request_tx, request_rx) = unbounded::<Envelope<T, U>>();
    let (reply_tx, reply_rx) = unbounded::<U>();
    let server = ChannelServer {
        request_rx,
        handler,
    };
    let client = ChannelClient {
        reply_tx,
        reply_rx,
        request_tx,
    };
    (client, server)
}
