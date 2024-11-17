use acts_channel::{create_seq, Message};
use serde::Serialize;

pub fn wrap_message<T: ?Sized + Serialize>(name: &str, value: &T) -> Message {
    Message {
        name: name.to_string(),
        seq: create_seq(),
        ack: None,
        data: Some(serde_json::to_vec(value).unwrap()),
    }
}
