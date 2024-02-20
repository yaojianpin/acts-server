use acts_channel::{
    model::{ModelInfo, ProcInfo, TaskInfo},
    ActionResult, Vars,
};
use chrono::prelude::*;
use prettytable::{row, Table};

pub fn process_result(name: &str, state: ActionResult) -> String {
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

            let mut table = Table::new();
            table.add_row(row!["id", "name", "version", "size", "time"]);
            for m in models {
                table.add_row(row![                    m.id,
                    m.name,
                    format!("{}", m.ver),
                    size(m.size),
                    local_time(m.time)]);
            }
            table.printstd();
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
            let mut table = Table::new();
            table.add_row(row!["pid", "name", "model id", "state", "start time"]);
            for p in procs {
                table.add_row(row![
                    p.id,
                    p.name,
                    p.mid,
                    p.state,
                    local_time(p.start_time)]);
            }
            table.printstd();
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
            let mut table = Table::new();
            table.add_row(row!["type", "tid", "name", "nid", "state", "start time", "end time"]);
            for p in tasks {
                table.add_row(row![                    
                    p.r#type,
                    p.id,
                    p.name,
                    p.node_id,
                    p.action_state,
                    local_time(p.start_time),
                    local_time(p.end_time)]);
            }
            table.printstd();
        }
        "start" | "submit" => {
            let vars = Vars::from_prost(&state.data.unwrap());
            let pid = vars.get("pid").unwrap();
            result.push_str(&format!("pid={pid}"));
        }
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
        "proc" => {
            let vars = Vars::from_prost(&state.data.unwrap()).into_inner();
            if let Some(proc) = vars.get("proc") {
                if let Some(map) = proc.as_object() {
                    for (key, v) in map {
                        if key == "tasks" {
                            continue;
                        }
                        result.push_str(&format!("{}:{}\n", key, v));
                    }
                    result.push_str("tasks:\n");
                    if let Some(v) = map.get("tasks") {
                        let text = v.as_str().unwrap();
                        result.push_str(&format!("{}\n", text));
                    }
                }
            }
        }
        "task" => {
            let vars = Vars::from_prost(&state.data.unwrap()).into_inner();
            if let Some(proc) = vars.get("task") {
                if let Some(map) = proc.as_object() {
                    for (key, v) in map {
                        result.push_str(&format!("{}:{}\n", key, v));
                    }
                }
            }
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
