use once_cell::sync::Lazy;
use std::collections::HashMap;

const ENV: &str = r#"env <op> [key] [value] [value-type]
    op: command with set, get, ls
            set: set key and value.
            get: get by key name
            ls: list all env values
            json: show in json format
    key: env key with string type
    value: env value
    value-type: value type with string, int, float and json, the default type is string
"#;

const RM: &str = r#"rm <proc|model> <key>
    proc <pid>: use to remove proc by pid
    model <mid>: use to remove model by mid
"#;

const SUB: &str = r#"sub <client_id> [kind] [event] [nkind] [topic]
    subscribe server message
    kind, event, nkind and topic are all support glob string

    client_id:  client id
    kind: message kind in task, act*, notice.
    event: message event in init, complete, error, cancel, abort and back.
    nkind: message node kind in workflow, job, branch and step
    topic: message topic which is defined in workflow model topic attribute.

    for examples:
    1. sub all messages:
    sub  1
    2. sub all act messages:
    sub 1 act*
    3. sub init and complete messages
    sub 1 * {init,complete}
    4. sub workflow messages with init, complete and error.
    sub 1 * {init,error,complete} workflow
    5. sub all messages that the topic starts with abc
    sub 1 * * * abc*

"#;

const MODELS: &str = r#"models [count]
    query the current deployed models
    
    count: expect to load the max model count
"#;

const MODEL: &str = r#"model <mid>
    query the model data
"#;

const PROCS: &str = r#"procs [count]
    query the current running procs

    count: expect to load the max proc count
"#;

const PROC: &str = r#"proc <pid>
    query the proc data
"#;

const TASKS: &str = r#"tasks <pid>
    query the proc tasks

    pid: the proc id
"#;

const TASK: &str = r#"task <pid> <tid>
    query the task data
"#;

const ACTS: &str = r#"acts <pid> <tid>
    query the proc acts

    pid: proc id
    tid: task id
"#;

const DEPLOY: &str = r#"deploy <path>
    deploy a workflow

    path: yml model local file path
"#;

const START: &str = r#"start <mid>
    start a workflow

    mid: workflow model id
"#;

const SUBMIT: &str = r#"submit <mid>
    submit data

    mid: model id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const BACK: &str = r#"back <pid> <aid>
    back to the history task

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const CANCEL: &str = r#"cancel <pid> <aid>
    cancel the act that is completed before

    pid: proc id
    aid: act id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const ABORT: &str = r#"abort <pid> <aid>
    abort the workflow

    pid: proc id
    aid: act id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const COMPLETE: &str = r#"abort <pid> <aid>
    complete the act

    pid: proc id
    aid: act id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const UPDATE: &str = r#"update <pid> <aid>
    update the variables

    pid: proc id
    aid: act id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

pub const MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("env", ENV),
        ("sub", SUB),
        ("models", MODELS),
        ("model", MODEL),
        ("rm", RM),
        ("procs", PROCS),
        ("proc", PROC),
        ("tasks", TASKS),
        ("task", TASK),
        ("acts", ACTS),
        ("deploy", DEPLOY),
        ("start", START),
        ("submit", SUBMIT),
        ("back", BACK),
        ("cancel", CANCEL),
        ("abort", ABORT),
        ("complete", COMPLETE),
        ("update", UPDATE),
    ])
});

pub fn cmd(name: &str) -> &str {
    match MAP.get(name) {
        Some(text) => text,
        None => "command not found",
    }
}

pub fn all() -> String {
    let mut ret = String::new();

    ret.push_str(r#"usage: <cmd> <args>\n\n"#);

    for (_, v) in MAP.iter() {
        ret.push_str(v);
        ret.push_str("\n");
    }

    ret
}
