use acts_channel::{
    model::{MessageInfo, ModelInfo, PackageInfo, ProcInfo, TaskInfo},
    ActionResult, Vars,
};
use chrono::prelude::*;
use prettytable::{row, Table};

pub fn process_result(name: &str, state: ActionResult) -> String {
    let mut result = String::new();
    match name {
        "models" => {
            let vars = Vars::from_prost(&state.data);
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
                table.add_row(row![
                    m.id,
                    m.name,
                    format!("{}", m.ver),
                    size(m.size),
                    local_time(m.time)
                ]);
            }
            table.printstd();
        }
        "procs" => {
            let vars = Vars::from_prost(&state.data);
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
                table.add_row(row![p.id, p.name, p.mid, p.state, local_time(p.start_time)]);
            }
            table.printstd();
        }
        "tasks" => {
            let vars = Vars::from_prost(&state.data);
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
            table.add_row(row![
                "type",
                "tid",
                "name",
                "nid",
                "state",
                "start time",
                "end time"
            ]);
            for p in tasks {
                table.add_row(row![
                    p.r#type,
                    p.id,
                    p.name,
                    p.nid,
                    p.state,
                    local_time(p.start_time),
                    local_time(p.end_time)
                ]);
            }
            table.printstd();
        }
        "packages" => {
            let vars = Vars::from_prost(&state.data);
            let procs = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<PackageInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            let mut table = Table::new();
            table.add_row(row!["id", "name", "size", "create time", "update time"]);
            for p in procs {
                table.add_row(row![
                    p.id,
                    p.name,
                    size(p.size),
                    local_time(p.create_time),
                    local_time(p.update_time)
                ]);
            }
            table.printstd();
        }
        "messages" => {
            let vars = Vars::from_prost(&state.data);
            let messages = vars
                .get(name)
                .map(|v| {
                    let mut arr: Vec<MessageInfo> = Vec::new();
                    for info in v.as_array().unwrap() {
                        arr.push(info.into())
                    }

                    arr
                })
                .unwrap();
            let mut table = Table::new();
            table.add_row(row![
                "type",
                "id",
                // "name",
                // "pid",
                "tid",
                "state",
                "key",
                // "tag",
                "retries",
                "status",
                // "inputs",
                // "outputs",
                "create time",
                "update time"
            ]);
            for p in messages {
                table.add_row(row![
                    p.r#type,
                    p.id,
                    // p.name,
                    // p.pid,
                    p.tid,
                    p.state,
                    p.key,
                    // p.tag,
                    p.retry_times,
                    p.status,
                    // p.inputs,
                    // p.outputs,
                    local_time(p.create_time),
                    local_time(p.update_time)
                ]);
            }
            table.printstd();
        }
        "start" | "submit" => {
            let vars = Vars::from_prost(&state.data);
            let pid = vars.get("pid").unwrap();
            result.push_str(&format!("pid={pid}"));
        }
        "model" => {
            let vars = Vars::from_prost(&state.data);
            let model = vars
                .get(name)
                .map(|v| {
                    let model: ModelInfo = v.into();
                    model
                })
                .unwrap();
            result.push_str(&model.data);
            result.push_str("\n");
        }
        "proc" => {
            let vars = Vars::from_prost(&state.data).into_inner();
            if let Some(v) = vars.get(name) {
                result.push_str(&serde_json::to_string_pretty(v).unwrap());
            }
        }
        "task" => {
            let vars = Vars::from_prost(&state.data).into_inner();
            if let Some(v) = vars.get(name) {
                result.push_str(&serde_json::to_string_pretty(v).unwrap());
            }
        }
        "message" => {
            let vars = Vars::from_prost(&state.data);
            let data = vars
                .get(name)
                .map(|v| serde_json::to_string_pretty(v).unwrap())
                .unwrap();
            result.push_str(&data);
            result.push_str("\n");
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
