use std::error::Error;
use std::sync::Arc;

use crate::actor::websocket::SocketId;
use tokio::sync::mpsc::{self, error::SendError};

use super::{delivery_service, websocket, ChatMessage};

struct User {
    delivery_service: delivery_service::Handle,
    name: Arc<str>,
    receiver: mpsc::Receiver<Message>,
    sockets: Vec<websocket::Handle>,
}

enum Message {
    AddSocket(websocket::Handle),
    ProcessSocketMessage(ChatMessage),
    ReceiveMessage(ChatMessage),
    RemoveSocket(SocketId),
}

async fn run_actor(mut actor: User) {
    while let Some(message) = actor.receiver.recv().await {
        match message {
            Message::AddSocket(socket) => {
                actor.sockets.push(socket);
            }
            Message::ProcessSocketMessage(message) => {
                //TODO send message to all other connected sockets for this user
                // Send message to the user it is intended for through delivery service
                let result = actor.delivery_service.send_message(message).await;
                let Err(error) = result else {
                    continue;
                };
                tracing::error!("Error sending message to delivery service: {:?}", error);
            }
            Message::ReceiveMessage(message) => {
                // Creating a reference that is easier to clone for each loop iteration
                let reference = Arc::new(message);
                //TODO this can easily be parallelized as it is fire and forget
                for socket in &actor.sockets {
                    let result = socket.send_message(reference.clone()).await;
                    if let Err(error) = result {
                        tracing::error!("Error sending message to socket: {}", error);
                    }
                }
            }
            Message::RemoveSocket(handle_id) => {
                // Not using retain as it would need to go through all elements, and we can be fairly
                // sure that the socket is only once in the list. Meaning after it was found the
                // iterations would be useless.
                let position = actor
                    .sockets
                    .iter()
                    .position(|handle| handle.id == handle_id);
                let Some(position) = position else {
                    // This is weird
                    tracing::warn!("Socket not found for deletion");
                    continue;
                };

                let _ = actor.sockets.remove(position);

                // If the socket is the last one, we can remove the user from the delivery service
                if actor.sockets.is_empty() {
                    let result = actor.delivery_service.remove_user(actor.name.clone()).await;
                    // Shut down
                    if let Err(error) = result {
                        tracing::error!("Error removing user from delivery service: {:?}", error);
                    }
                    // Shut down anyway?
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Handle {
    sender: mpsc::Sender<Message>,
}

impl Handle {
    pub(crate) fn new(name: Arc<str>, delivery_service: delivery_service::Handle) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let actor = User {
            delivery_service,
            name,
            receiver,
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

    pub(super) async fn process_socket_message(
        &self,
        message: ChatMessage,
    ) -> Result<(), impl Error + Send + Sync> {
        self.sender
            .send(Message::ProcessSocketMessage(message))
            .await
    }

    pub(super) async fn receive_message(&self, message: ChatMessage) -> Result<(), impl Error> {
        self.sender.send(Message::ReceiveMessage(message)).await
    }

    pub(super) async fn remove_socket(&self, socket_id: SocketId) -> Result<(), impl Error> {
        self.sender.send(Message::RemoveSocket(socket_id)).await
    }
}
