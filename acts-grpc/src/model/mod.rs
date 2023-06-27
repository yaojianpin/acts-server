mod info;
mod message;
mod state;

use serde_json::Value;

pub type ActValue = Value;
pub use info::{ActInfo, ModelInfo, ProcInfo, TaskInfo};
pub use message::{
    ActionMessage, CandidateMessage, Message, NoticeMessage, SomeMessage, TaskMessage, UserMessage,
};
pub use state::ActionState;
