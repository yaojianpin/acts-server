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

const SUB: &str = r#"sub <client_id> [type] [state] [tag] [key]
    subscribe server message
    type, state and tag are all support glob string

    client_id:  client id
    type: message types are in workflow, job, step, branch and act.
    state: message state in created, completed, error, cancelled, aborted, skipped and backed.
    tag: message tag which is defined in workflow model tag attribute.
    key: message key

    for examples:
    1. sub all messages:
    sub  1
    2. sub all act messages:
    sub 1 act
    3. sub created and complete messages
    sub 1 * {created,completed}
    4. sub all messages that the tag starts with abc
    sub 1 * * abc*
    5. sub all messages that the key starts with 123
    sub 1 * * * 123*
"#;

const MODELS: &str = r#"models [count]
    query the current deployed models
    
    count: expect to load the max model count
"#;

const MODEL: &str = r#"model <mid> [fmt]
    query the model data
    mid: model id
    fmt: display format with text|json|tree
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

const DEPLOY: &str = r#"deploy <path>
    deploy a workflow

    path: yml model local file path
"#;

const START: &str = r#"start <mid>
    start a workflow

    mid: workflow model id
"#;

const SUBMIT: &str = r#"submit <pid> <tid>
    submit an action

    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const BACK: &str = r#"back <pid> <tid>
    back to the history task

    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const CANCEL: &str = r#"cancel <pid> <tid>
    cancel the act that is completed before

    pid: proc id
    tid: task id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const ABORT: &str = r#"abort <pid> <tid>
    abort the workflow

    pid: proc id
    tid: task id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const COMPLETE: &str = r#"abort <pid> <tid>
    complete the act

    pid: proc id
    tid: task id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const SKIP: &str = r#"skip <pid> <tid>
    skip the action

    pid: proc id
    tid: task id
    
    nodes: this command can execute with extra options
    the options is from the env which is created through env command
"#;

const ERROR: &str = r#"error <pid> <tid>
    set an action as error

    pid: proc id
    tid: task id
    
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
        ("deploy", DEPLOY),
        ("start", START),
        ("submit", SUBMIT),
        ("back", BACK),
        ("cancel", CANCEL),
        ("abort", ABORT),
        ("complete", COMPLETE),
        ("skip", SKIP),
        ("error", ERROR),
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
