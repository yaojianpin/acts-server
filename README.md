# acts-server
create a acts workflow server based on  [`acts`](https://github.com/yaojianpin/acts) lib

# acts-cli
a command client for acts-server

the supported commands as follows:

* deploy a workflow
```
deploy <path>
    
    path: yml model local file path
```

* subscribe server message
```
sub <client_id> [type] [state] [tag] [key]
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
```
* query model data
```
model <mid> [tree]
```

* remove model
```
rm <key>
    key: model id
```

* start a workflow
```
start <mid>
    mid: workflow model id
```

* list a proc's tasks
```
tasks <pid>
    pid: the proc id
```

* query task data
```
task <pid> <tid>
```

env <op> [key] [value] [value-type]
    op: command with set, get, ls
            set: set key and value.
            get: get by key name
            ls: list all env values
            json: show in json format
    key: env key with string type
    value: env value
    value-type: value type with string, int, float and json, the default type is string



* query the deployed models
```
models [count]
    count: expect to load the max model count
```

* query a proc's information
```
proc <pid> [tree|json]
    pid: proc id
    tree: show the proc tasks in tree
    json show the proc tasks in json format
```

* push the act to a step
```
push <pid> <tid>
    pid: proc id
    tid: step task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* remove an act
```
remove <pid> <tid>
    pid: proc id
    tid: step task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* submit an act
```
submit <pid> <tid>
    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

*  back to the history task
```
back <pid> <tid>
    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```
* abort the workflow
```
abort <pid> <tid>
    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* query the current running procs
```
procs [count]
    count: expect to load the max proc count
```
* complete the act
```
abort <pid> <tid>
    pid: proc id
    tid: task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* cancel the act which is completed before
```
cancel <pid> <tid>
    pid: proc id
    tid: act task id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```



