# acts-server
create a acts workflow server based on  acts lib

# acts-cli
a command client for acts-server

the supported commands as follows:

* query model data
```
model <mid>
```

* query task data
```
task <pid> <tid>
```

* deploy a workflow
```
deploy <path>
    
    path: yml model local file path
```

* subscribe server message
```
sub <client_id> [kind] [event] [nkind] [topic]
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

* update the proc variables
```
update <pid> <aid>
    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
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

* submit data
```
submit <mid>
    mid: model id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* query the deployed models
```
models [count]
    count: expect to load the max model count
```

*  query a proc's acts
```
acts <pid> <tid>
    pid: proc id
    tid: task id
```

* query a proc's information
```
proc <pid>
```

*  back to the history task
```
back <pid> <aid>
    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```
* abort the workflow
```
abort <pid> <aid>
    pid: proc id
    aid: act id

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
abort <pid> <aid>
    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

* cancel the act which is completed before
```
cancel <pid> <aid>
    pid: proc id
    aid: act id

    nodes: this command can execute with extra options
    the options is from the env which is created through env command
```

