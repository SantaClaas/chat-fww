use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::OffsetDateTime;

pub(super) mod delivery_service;
pub(super) mod user;
pub(super) mod websocket;

#[derive(Serialize, Deserialize, Debug)]
pub(in crate::actor) struct ChatMessage {
    pub(super) to: Arc<str>,
    from: String,
    text: String,
    #[serde(
        with = "time::serde::timestamp::milliseconds",
        rename = "time"
    )]
    time_utc: OffsetDateTime,
}
