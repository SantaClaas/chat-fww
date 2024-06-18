use crate::actor::{user, ChatMessage};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

enum Message {
    //TODO handle case where a user actor is not found
    SendMessage(ChatMessage),
    GetOrInsertUser(Arc<str>, oneshot::Sender<user::Handle>),
    GetUsers(oneshot::Sender<Arc<[Arc<str>]>>),
    RemoveUser(Arc<str>),
}

/// The delivery service actor is responsible for sending messages between user actors.
/// The alternative would be for user actors to have references to all other user actors
/// and send messages to them directly, but this would be a fully connected mesh topology where the
/// handles across all actors in memory would grow quadratically (meaning A LOT...
/// See: https://www.wevolver.com/article/mesh-topology).
/// Instead, we use a start topology with the delivery service actor in the center.
/// Currently, the delivery service also acts as kind of registry for user actors.
struct DeliveryService {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    users_by_name: HashMap<Arc<str>, user::Handle>,
}

impl DeliveryService {
    fn get_handle(&self) -> Handle {
        self.sender.clone().into()
    }
}

async fn run_actor(mut actor: DeliveryService) {
    while let Some(message) = actor.receiver.recv().await {
        tracing::debug!("Processing message");
        match message {
            Message::SendMessage(message) => {
                let receiver = actor.users_by_name.get(&message.recipient);
                let Some(receiver) = receiver else {
                    //TODO send error to user if receiver is not found
                    tracing::error!("User not found");
                    continue;
                };

                let result = receiver.receive_message(message).await;
                if let Err(error) = result {
                    tracing::error!("Error sending message to user: {}", error);
                }
            }
            Message::GetOrInsertUser(user_name, respond) => {
                let entry = actor.users_by_name.get(user_name.as_ref());

                let user = entry.cloned().unwrap_or_else(|| {
                    let user = user::Handle::new(user_name.clone(), actor.get_handle());
                    actor.users_by_name.insert(user_name.clone(), user.clone());
                    user
                });

                let result = respond.send(user.clone());
                if result.is_ok() {
                    continue;
                }
                tracing::error!("Error sending user handle back");
            }
            Message::GetUsers(responder) => {
                let users = actor.users_by_name.keys().cloned().collect();
                let result = responder.send(users);
                if result.is_ok() {
                    continue;
                }
                tracing::error!("Error sending users back");
            }
            Message::RemoveUser(name) => {
                let _ = actor.users_by_name.remove(&name);
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Handle {
    sender: mpsc::Sender<Message>,
}

impl Default for Handle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HandleError {
    #[error("Error sending message to actor")]
    SendError(#[from] mpsc::error::SendError<Message>),
    #[error("Error receiving answer with user from actor")]
    ReceiveError(#[from] oneshot::error::RecvError),
}

impl Handle {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let delivery_service = DeliveryService {
            receiver,
            users_by_name: HashMap::new(),
            sender: sender.clone(),
        };

        tokio::spawn(run_actor(delivery_service));

        Self { sender }
    }

    pub(crate) async fn get_or_insert(
        &self,
        user_name: Arc<str>,
    ) -> Result<user::Handle, HandleError> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Message::GetOrInsertUser(user_name, sender))
            .await?;

        let handle = receiver.await?;

        Ok(handle)
    }

    pub(crate) async fn get_users(&self) -> Result<Arc<[Arc<str>]>, HandleError> {
        let (sender, receiver) = oneshot::channel();
        self.sender.send(Message::GetUsers(sender)).await?;
        let users = receiver.await?;
        Ok(users)
    }

    pub(super) async fn send_message(&self, message: ChatMessage) -> Result<(), impl Error> {
        self.sender.send(Message::SendMessage(message)).await
    }

    pub(super) async fn remove_user(&self, name: Arc<str>) -> Result<(), impl Error> {
        self.sender.send(Message::RemoveUser(name)).await
    }
}

impl From<mpsc::Sender<Message>> for Handle {
    fn from(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}
