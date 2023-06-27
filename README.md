# acts-server
create a acts workflow server based on  acts lib

# acts-cli
a command client for acts-server

the supported commands as follows:
model <mid>
    query the model data

task <pid> <tid>
    query the task data

deploy <path>
    deploy a workflow

    path: yml model local file path

sub <client_id> [kind] [event] [nkind] [topic]
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


rm <proc|model> <key>
    proc <pid>: use to remove proc by pid
    model <mid>: use to remove model by mid

start <mid>
    start a workflow

    mid: workflow model id

tasks <pid>
    query the proc tasks

    pid: the proc id

update <pid> <aid>
    update the variables

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command

env <op> [key] [value] [value-type]
    op: command with set, get, ls
            set: set key and value.
            get: get by key name
            ls: list all env values
            json: show in json format
    key: env key with string type
    value: env value
    value-type: value type with string, int, float and json, the default type is string

submit <mid>
    submit data

    mid: model id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command

models [count]
    query the current deployed models

    count: expect to load the max model count

acts <pid> <tid>
    query the proc acts

    pid: proc id
    tid: task id

proc <pid>
    query the proc data

back <pid> <aid>
    back to the history task

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command

abort <pid> <aid>
    abort the workflow

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command

procs [count]
    query the current running procs

    count: expect to load the max proc count

abort <pid> <aid>
    complete the act

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command

cancel <pid> <aid>
    cancel the act that is completed before

    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command


