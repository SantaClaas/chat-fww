use std::sync::Arc;

use tokio::sync::mpsc::{self, error::SendError};

use super::websocket;

struct User {
    name: Arc<str>,
    reveiver: mpsc::Receiver<Message>,
    sockets: Vec<websocket::Handle>,
}

enum Message {
    AddSocket(websocket::Handle),
}

#[derive(Clone)]
pub(crate) struct Handle {
    sender: mpsc::Sender<Message>,
}

async fn run_actor(mut actor: User) {
    while let Some(message) = actor.reveiver.recv().await {
        match message {
            Message::AddSocket(socket) => {
                actor.sockets.push(socket);
            }
        }
    }
}

impl Handle {
    pub(crate) fn new(name: Arc<str>) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = User {
            name,
            reveiver: receiver,
            // We know from context that a user will be only created if there is at least one socket
            sockets: Vec::with_capacity(1),
        };

        tokio::spawn(run_actor(actor));

        Self { sender }
    }

    pub(crate) async fn add_socket(
        &self,
        socket: websocket::Handle,
    ) -> Result<(), impl std::error::Error> {
        self.sender.send(Message::AddSocket(socket)).await
    }
}
