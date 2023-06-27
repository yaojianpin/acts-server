use serde::{Deserialize, Serialize};
type Vars = serde_json::Map<String, serde_json::Value>;

#[derive(Deserialize, Serialize, Clone)]
pub struct ActionState {
    pub start_time: i64,
    pub end_time: i64,
    pub outputs: Vars,
}

impl ActionState {
    /// Get the workflow output vars
    pub fn outputs(&self) -> &Vars {
        &self.outputs
    }
    /// How many time(million seconds) did a workflow cost
    pub fn cost(&self) -> i64 {
        self.end_time - self.start_time
    }
}
