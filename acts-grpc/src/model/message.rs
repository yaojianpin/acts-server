use crate::WorkflowMessage;

use super::ActValue;
use serde::{Deserialize, Serialize};
type Vars = serde_json::Map<String, serde_json::Value>;
// #[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
// pub enum EventAction {
//     #[default]
//     Create,
//     Next,
//     Submit,
//     Skip,
//     Back,
//     Cancel,
//     Abort,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Default)]
// pub enum ActKind {
//     #[default]
//     Action = 0,
//     Some,
//     Candidate,
//     User,
// }

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub enum MessageKind {
//     Task,
//     Act(ActKind),
//     Notice,
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub kind: String,
    pub event: String,
    pub mid: String,
    pub topic: String,
    pub nkind: String,
    pub nid: String,
    pub pid: String,
    pub tid: String,
    pub key: Option<String>,
    pub vars: Vars,
}

#[derive(Clone, Debug)]
pub struct TaskMessage<'a> {
    pub event: &'a str,
    pub pid: &'a str,
    pub tid: &'a str,
    pub vars: &'a Vars,
}

#[derive(Clone, Debug)]
pub struct UserMessage<'a> {
    pub event: &'a str,
    pub pid: &'a str,
    pub aid: &'a str,
    pub uid: &'a str,
    pub vars: &'a Vars,
}

#[derive(Clone, Debug)]
pub struct CandidateMessage<'a> {
    pub event: &'a str,
    pub pid: &'a str,
    pub aid: &'a str,
    pub cands: ActValue,
    pub matcher: &'a str,
    pub vars: &'a Vars,
}

#[derive(Clone, Debug)]
pub struct ActionMessage<'a> {
    pub event: &'a str,
    pub pid: &'a str,
    pub aid: &'a str,
    pub action: &'a str,
    pub vars: &'a Vars,
}

#[derive(Clone, Debug)]
pub struct NoticeMessage<'a> {
    pub pid: &'a str,
    pub key: &'a str,
    pub vars: &'a Vars,
}

#[derive(Clone, Debug)]
pub struct SomeMessage<'a> {
    pub event: &'a str,
    pub pid: &'a str,
    pub aid: &'a str,
    pub some: &'a str,
    pub vars: &'a Vars,
}

impl Message {
    // pub fn new(kind: MessageKind, event: &EventAction, pid: &str, key: &str, vars: &Vars) -> Self {
    //     Self {
    //         kind,
    //         event: event.clone(),
    //         pid: pid.to_string(),
    //         key: key.to_string(),
    //         vars: vars.clone(),
    //     }
    // }

    pub fn as_task_message(&self) -> Option<TaskMessage> {
        match self.kind.as_str() {
            "task" => Some(TaskMessage {
                pid: &self.pid,
                event: &self.event,
                tid: &self.tid,
                vars: &self.vars,
            }),
            _ => None,
        }
    }

    pub fn as_user_message(&self) -> Option<UserMessage> {
        match self.kind.as_str() {
            "act:user" => Some(UserMessage {
                pid: &self.pid,
                event: &self.event,
                aid: self.key.as_ref().unwrap(),
                vars: &self.vars,
                uid: self.vars.get("owner").unwrap().as_str().unwrap(),
            }),
            _ => None,
        }
    }
    pub fn as_action_message(&self) -> Option<ActionMessage> {
        match self.kind.as_str() {
            "act:action" => Some(ActionMessage {
                pid: &self.pid,
                event: &self.event,
                aid: self.key.as_ref().unwrap(),
                vars: &self.vars,
                action: self.vars.get("action").unwrap().as_str().unwrap(),
            }),
            _ => None,
        }
    }

    pub fn as_notice_message(&self) -> Option<NoticeMessage> {
        match self.kind.as_str() {
            "act:notice" => Some(NoticeMessage {
                pid: &self.pid,
                vars: &self.vars,
                key: self.key.as_ref().unwrap(),
            }),
            _ => None,
        }
    }

    pub fn as_canidate_message(&self) -> Option<CandidateMessage> {
        match self.kind.as_str() {
            "act:candidate" => {
                let cand_value = self.vars.get("sub_cands").unwrap();
                let cands = cand_value.clone().into();
                let matcher = self.vars.get("sub_matcher").unwrap().as_str().unwrap();
                Some(CandidateMessage {
                    pid: &self.pid,
                    event: &self.event,
                    aid: self.key.as_ref().unwrap(),
                    vars: &self.vars,
                    cands,
                    matcher,
                })
            }
            _ => None,
        }
    }

    pub fn as_some_message(&self) -> Option<SomeMessage> {
        match self.kind.as_str() {
            "act:some" => Some(SomeMessage {
                pid: &self.pid,
                event: &self.event,
                aid: self.key.as_ref().unwrap(),
                vars: &self.vars,
                some: self.vars.get("some").unwrap().as_str().unwrap(),
            }),
            _ => None,
        }
    }
}

impl From<WorkflowMessage> for Message {
    fn from(v: WorkflowMessage) -> Self {
        let vars = crate::Vars::from_prost(&v.vars.unwrap());
        Self {
            kind: v.kind,
            event: v.event,
            mid: v.mid,
            topic: v.topic,
            nkind: v.nkind,
            nid: v.nid,
            pid: v.pid,
            tid: v.tid,
            key: v.key,
            vars: vars.json_vars(),
        }
    }
}
