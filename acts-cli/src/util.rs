use acts_grpc::{
    model::{ActInfo, ModelInfo, ProcInfo, TaskInfo},
    ActionState, Vars,
};
use chrono::prelude::*;

pub fn process_result(name: &str, state: ActionState) -> String {
    let mut result = String::new();
    match name {
        "models" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let models = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<ModelInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            result.push_str(&format!(
                "{:36}{:20}{:10}{:10}{:20}\n",
                "id", "name", "version", "size", "time"
            ));
            for m in models {
                result.push_str(&format!(
                    "{:36}{:20}{:10}{:10}{:20}\n",
                    m.id,
                    m.name,
                    format!("{}", m.ver),
                    size(m.size),
                    local_time(m.time)
                ));
            }
        }
        "procs" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let procs = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<ProcInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            result.push_str(&format!(
                "{:48}{:20}{:36}{:16}{:20}\n",
                "pid", "name", "model id", "state", "start time"
            ));
            for p in procs {
                result.push_str(&format!(
                    "{:48}{:20}{:36}{:16}{:20}\n",
                    p.pid,
                    p.name,
                    p.mid,
                    p.state,
                    local_time(p.start_time)
                ));
            }
        }
        "tasks" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let tasks = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<TaskInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            result.push_str(&format!(
                "{:12}{:16}{:16}{:16}{:20}{:20}\n",
                "kind", "tid", "nid", "state", "start time", "end time",
            ));
            for p in tasks {
                result.push_str(&format!(
                    "{:12}{:16}{:16}{:16}{:20}{:20}\n",
                    p.kind,
                    p.tid,
                    p.nid,
                    p.state,
                    local_time(p.start_time),
                    local_time(p.end_time)
                ));
            }
        }
        "acts" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let acts = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<ActInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            result.push_str(&format!(
                "{:16}{:16}{:16}{:16}{:20}{:20}\n",
                "kind", "aid", "tid", "state", "start time", "end time",
            ));
            for act in acts {
                result.push_str(&format!(
                    "{:16}{:16}{:16}{:16}{:20}{:20}\n",
                    act.kind,
                    act.aid,
                    act.tid,
                    act.state,
                    local_time(act.start_time),
                    local_time(act.end_time)
                ));
            }
        }
        "start" | "submit" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let pid = vars.get("pid").unwrap();
            result.push_str(&format!("pid={pid}"));
        }
        // "abort" | "back" | "cancel" | "complete" => {}
        "model" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let model = vars
                .get(name)
                .map(|v| {
                    let model: ModelInfo = v.into();

                    model
                })
                .unwrap();
            result.push_str(&model.model);
            result.push_str("\n");
        }
        "proc" | "task" => {
            let vars: Vars = Vars::from_prost(&state.data.unwrap());
            result.push_str(&format!("{}", vars));
        }
        _ => {}
    };

    // print the elapsed
    let cost = state.end_time - state.start_time;
    result.push_str(&format!("(elapsed {cost}ms)"));

    result
}

fn local_time(millis: i64) -> String {
    if millis == 0 {
        return "(nil)".to_string();
    }
    match Local.timestamp_millis_opt(millis) {
        chrono::LocalResult::Single(dt) => format!("{}", dt.format("%Y-%m-%d %H:%M:%S")),
        _ => "".to_string(),
    }
}

fn size(bits: u32) -> String {
    let mut ret = String::new();
    if bits < 1024 {
        ret.push_str(&format!("{}b", bits));
    } else {
        let kb = bits / 1024;
        if kb < 1024 {
            ret.push_str(&format!("{}kb", kb));
        } else {
            let m = kb / 1024;
            if m < 1024 {
                ret.push_str(&format!("{}m", m));
            }
        }
    }

    ret
}
