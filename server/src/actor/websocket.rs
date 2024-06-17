use ::axum::extract::ws::Message as WebSocketMessage;
use axum::extract::ws as axum;
use tokio::sync::mpsc;

use super::user;

struct WebSocket {
    socket: axum::WebSocket,
    receiver: mpsc::Receiver<Message>,
    /// The user that is connected through this websocket
    user: user::Handle,
}

impl WebSocket {
    fn handle_socket_message(&self, message: WebSocketMessage) {}
}

enum Message {}

#[derive(Clone)]
pub(crate) struct Handle {
    sender: mpsc::Sender<Message>,
}

async fn run_actor(mut actor: WebSocket) {
    loop {
        tokio::select! {
            Some(message) = actor.receiver.recv() => {
                todo!()
            },
            // Stop actor on error
            Some(Ok(message)) = actor.socket.recv() => {
                tracing::info!("Received message: {:?}", message);
                if let WebSocketMessage::Close(_) = message {
                    //TODO remove socket from user to clean up or we have a memory leak
                    tracing::info!("Closing websocket");
                    break;
                }
            },
            else => break,
        }
    }
}

impl Handle {
    pub(crate) fn new(socket: axum::WebSocket, user: user::Handle) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let socket = WebSocket {
            socket,
            receiver,
            user,
        };

        tokio::spawn(run_actor(socket));

        Self { sender }
    }
}
