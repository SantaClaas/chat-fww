use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;

pub(super) mod delivery_service;
pub(super) mod user;
pub(super) mod websocket;

#[derive(Serialize, Deserialize, Debug)]
pub(in crate::actor) struct ChatMessage {
    pub(super) recipient: Arc<str>,
    sender: String,
    text: String,
    #[serde(
        with = "time::serde::timestamp::milliseconds",
    )]
    time_utc: OffsetDateTime,
}
