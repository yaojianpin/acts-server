use serde::{Deserialize, Serialize};

macro_rules! unpack_value_string {
    ($value: ident, $name: expr) => {
        $value.get($name).unwrap().as_str().unwrap().to_string()
    };
}

macro_rules! unpack_value_number {
    ($value: ident, $name: expr) => {
        $value.get($name).unwrap().as_f64().unwrap()
    };
}

// macro_rules! unpack_value_bool {
//     ($value: ident, $name: expr) => {
//         $value.get($name).unwrap().as_bool().unwrap()
//     };
// }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcInfo {
    pub pid: String,
    pub name: String,
    pub mid: String,
    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskInfo {
    pub pid: String,
    pub tid: String,
    pub nid: String,
    pub kind: String,
    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActInfo {
    pub aid: String,
    pub kind: String,
    pub pid: String,
    pub tid: String,
    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub ver: u32,
    pub size: u32,
    pub time: i64,
    pub model: String,
    pub topic: String,
}

impl From<&serde_json::Value> for ModelInfo {
    fn from(value: &serde_json::Value) -> Self {
        Self {
            id: unpack_value_string!(value, "id"),
            name: unpack_value_string!(value, "name"),
            model: unpack_value_string!(value, "model"),
            ver: unpack_value_number!(value, "ver") as u32,
            size: unpack_value_number!(value, "size") as u32,
            time: unpack_value_number!(value, "time") as i64,
            topic: unpack_value_string!(value, "topic"),
        }
    }
}

impl From<&serde_json::Value> for ProcInfo {
    fn from(value: &serde_json::Value) -> Self {
        Self {
            pid: unpack_value_string!(value, "pid"),
            name: unpack_value_string!(value, "name"),
            mid: unpack_value_string!(value, "mid"),
            state: unpack_value_string!(value, "state"),
            start_time: unpack_value_number!(value, "start_time") as i64,
            end_time: unpack_value_number!(value, "end_time") as i64,
        }
    }
}

impl From<&serde_json::Value> for TaskInfo {
    fn from(value: &serde_json::Value) -> Self {
        Self {
            pid: unpack_value_string!(value, "pid"),
            tid: unpack_value_string!(value, "tid"),
            nid: unpack_value_string!(value, "nid"),
            kind: unpack_value_string!(value, "kind"),
            state: unpack_value_string!(value, "state"),
            start_time: unpack_value_number!(value, "start_time") as i64,
            end_time: unpack_value_number!(value, "end_time") as i64,
        }
    }
}

impl From<&serde_json::Value> for ActInfo {
    fn from(value: &serde_json::Value) -> Self {
        Self {
            aid: unpack_value_string!(value, "aid"),
            kind: unpack_value_string!(value, "kind"),
            pid: unpack_value_string!(value, "pid"),
            tid: unpack_value_string!(value, "tid"),
            state: unpack_value_string!(value, "state"),
            start_time: unpack_value_number!(value, "start_time") as i64,
            end_time: unpack_value_number!(value, "end_time") as i64,
        }
    }
}
