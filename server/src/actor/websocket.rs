use ::axum::extract::ws::Message as WebSocketMessage;
use axum::extract::ws as axum;
use nanoid::nanoid;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::mpsc;

use super::{user, ChatMessage};

enum Message {
    SendMessage(Arc<ChatMessage>),
    AddContact { name: Arc<str> },
    RemoveContact { name: Arc<str> },
    SynchronizeMessage { message: Arc<ChatMessage> },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    ChatMessage { message: Arc<ChatMessage> },
    AddUser { name: Arc<str> },
    RemoveUser { name: Arc<str> },
    SynchronizeMessage { message: Arc<ChatMessage> },
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct SocketId(Arc<str>);

struct WebSocket {
    id: SocketId,
    socket: axum::WebSocket,
    receiver: mpsc::Receiver<Message>,
    /// The user that is connected through this websocket
    user: user::Handle,
}

async fn process_socket_message(id: SocketId, user: &user::Handle, json: String) {
    //TODO validate message sender name and send back error if not
    let result = serde_json::from_str::<ChatMessage>(&json);
    let message = match result {
        Ok(message) => message,
        Err(error) => {
            tracing::error!("Error deserializing message: {:?}", error);
            return;
        }
    };

    let result = user.process_socket_message(id, message.into()).await;
    if let Err(error) = result {
        tracing::error!("Error processing message: {:?}", error);
    }
}

async fn send_to_socket(socket: &mut axum::WebSocket, message: ClientMessage) {
    let json = serde_json::to_string(&message);
    let result = match json {
        Ok(json) => socket.send(WebSocketMessage::Text(json)).await,
        Err(error) => {
            tracing::error!("Error serializing message: {:?}", error);
            return;
        }
    };

    if let Err(error) = result {
        tracing::error!("Error sending message through websocket: {:?}", error);
    }
}

async fn process_actor_message(socket: &mut axum::WebSocket, message: Message) {
    match message {
        Message::SendMessage(message) => {
            let message = ClientMessage::ChatMessage { message };
            send_to_socket(socket, message).await;
        }
        Message::AddContact { name } => {
            let message = ClientMessage::AddUser { name };
            send_to_socket(socket, message).await;
        }
        Message::RemoveContact { name } => {
            let message = ClientMessage::RemoveUser { name };
            send_to_socket(socket, message).await;
        }
        Message::SynchronizeMessage { message } => {
            let message = ClientMessage::SynchronizeMessage { message };
            send_to_socket(socket, message).await;
        }
    }
}

async fn run_actor(mut actor: WebSocket) {
    loop {
        tokio::select! {
            Some(message) = actor.receiver.recv() => process_actor_message(&mut actor.socket, message).await,
            // Stop actor on error
            Some(Ok(message)) = actor.socket.recv() => {
                match message {
                    WebSocketMessage::Close(_) => {
                        //TODO remove socket from user to clean up or we have a memory leak
                        tracing::info!("Closing websocket");
                        let result = actor.user.remove_socket(actor.id.clone()).await;
                        let Err(error) = result else { break };
                        tracing::error!("Error removing socket from user: {:?}", error);
                        break;
                    },
                    WebSocketMessage::Text(text) => process_socket_message(actor.id.clone(), &actor.user, text.into()).await,
                    other => tracing::error!("Unexpected message type: {:?}", other),
                }
            },
            else => break,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Handle {
    pub(super) id: SocketId,
    sender: mpsc::Sender<Message>,
}

impl Handle {
    pub(crate) fn new(socket: axum::WebSocket, user: user::Handle) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        // I'd prefer a simple index to a full string to identify a socket for a user but this is
        // simpler to set up right now. Feel free to find a better solution.
        let id = SocketId(nanoid!().into());

        let socket = WebSocket {
            id: id.clone(),
            socket,
            receiver,
            user,
        };

        tokio::spawn(run_actor(socket));

        Self { sender, id }
    }

    pub(super) async fn send_message(
        &self,
        message: Arc<ChatMessage>,
    ) -> Result<(), impl std::error::Error> {
        self.sender.send(Message::SendMessage(message)).await
    }

    pub(super) async fn add_contact(&self, name: Arc<str>) -> Result<(), impl std::error::Error> {
        self.sender.send(Message::AddContact { name }).await
    }

    pub(super) async fn remove_contact(&self, name: Arc<str>) -> Result<(), impl std::error::Error> {
        self.sender.send(Message::RemoveContact { name }).await
    }

    pub(super) async fn synchronize_message(&self, message: Arc<ChatMessage>) -> Result<(), impl std::error::Error> {
        self.sender.send(Message::SynchronizeMessage { message }).await
    }
}
