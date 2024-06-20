use std::error::Error;
use std::sync::Arc;

use crate::actor::websocket::SocketId;
use tokio::sync::mpsc::{self};

use super::{delivery_service, websocket, ChatMessage};

struct User {
    delivery_service: delivery_service::Handle,
    name: Arc<str>,
    receiver: mpsc::Receiver<Message>,
    sockets: Vec<websocket::Handle>,
}

enum Message {
    AddSocket(websocket::Handle),
    ProcessSocketMessage(
        SocketId,
        Arc<ChatMessage>,
    ),
    ReceiveMessage(Arc<ChatMessage>),
    RemoveSocket(SocketId),
    AddContact(Arc<str>),
    RemoveContact(Arc<str>),
}

async fn run_actor(mut actor: User) {
    while let Some(message) = actor.receiver.recv().await {
        match message {
            Message::AddSocket(socket) => {
                actor.sockets.push(socket);
            }
            Message::ProcessSocketMessage(source, message) => {
                // Synchronize message to all other connected sockets for this user
                for socket in &actor.sockets {
                    if socket.id == source {
                        continue;
                    }


                    tracing::debug!("Syncing message");
                    let result = socket.synchronize_message(message.clone()).await;
                    if let Err(error) = result {
                        tracing::error!("Error sending message to socket: {}", error);
                    }
                }

                // Send message to the user it is intended for through delivery service
                let result = actor.delivery_service.send_message(message).await;
                let Err(error) = result else {
                    continue;
                };
                tracing::error!("Error sending message to delivery service: {:?}", error);
            }
            Message::ReceiveMessage(message) => {
                // Creating a reference that is easier to clone for each loop iteration
                //TODO this can easily be parallelized as it is fire and forget
                for socket in &actor.sockets {
                    let result = socket.send_message(message.clone()).await;
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
                    tracing::debug!("All sockets closed, removing user from delivery service");
                    let result = actor.delivery_service.remove_user(actor.name.clone()).await;
                    // Shut down
                    if let Err(error) = result {
                        tracing::error!("Error removing user from delivery service: {:?}", error);
                    }
                    // Shut down anyway?
                    break;
                }
            }
            Message::AddContact(user_name) => {
                for socket in &actor.sockets {
                    let result = socket.add_contact(user_name.clone()).await;
                    if let Err(error) = result {
                        tracing::error!("Error adding user to socket: {}", error);
                    }
                }
            }
            Message::RemoveContact(user_name) => {
                for socket in &actor.sockets {
                    let result = socket.remove_contact(user_name.clone()).await;
                    if let Err(error) = result {
                        tracing::error!("Error removing user from socket: {}", error);
                    }
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
    ) -> Result<(), impl Error> {
        self.sender.send(Message::AddSocket(socket)).await
    }

    pub(super) async fn process_socket_message(
        &self,
        source: SocketId,
        message: Arc<ChatMessage>,
    ) -> Result<(), impl Error + Send + Sync> {
        self.sender
            .send(Message::ProcessSocketMessage(source, message))
            .await
    }

    pub(super) async fn receive_message(&self, message: Arc<ChatMessage>) -> Result<(), impl Error> {
        self.sender.send(Message::ReceiveMessage(message)).await
    }

    pub(super) async fn remove_socket(&self, socket_id: SocketId) -> Result<(), impl Error> {
        self.sender.send(Message::RemoveSocket(socket_id)).await
    }

    pub(super) async fn add_contact(&self, user_name: Arc<str>) -> Result<(), impl Error> {
        self.sender.send(Message::AddContact(user_name)).await
    }

    pub(super) async fn remove_contact(&self, user_name: Arc<str>) -> Result<(), impl Error> {
        self.sender.send(Message::RemoveContact(user_name)).await
    }
}
