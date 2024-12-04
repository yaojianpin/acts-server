# acts-server

[![Build](https://github.com/yaojianpin/acts-server/actions/workflows/build.yml/badge.svg)](https://github.com/yaojianpin/acts-server/actions?workflow=build)
[![Release](https://github.com/yaojianpin/acts-server/actions/workflows/release.yml/badge.svg)](https://github.com/yaojianpin/acts-server/actions?workflow=release)

Create a acts workflow server based on [`acts`](https://github.com/yaojianpin/acts) lib

# acts-server

Download server file from [`Releases`](https://github.com/yaojianpin/acts-server/releases) and start it

# acts-cli

a command client for acts-server

the supported commands as follows:

```console
cli for acts-server

Usage: acts-cli.exe [OPTIONS]

Options:
      --host <HOST>
  -p, --port <PORT>
  -h, --help         Print help
```

after started `acts-cli`, run <COMMAND> to interacte with `acts-server`

```console
Usage: <COMMAND>

Commands:
  model    execute model commands
  package  execute package commands
  proc     execute proc commands
  task     execute task commands
  message  execute message commands
  act      execute act commands
  exit     exit the cli
  help     Print this message or the help of the given subcommand(s)
```

# client channels

- rust https://github.com/yaojianpin/acts-channel
- python https://github.com/yaojianpin/acts-channel-py
- go https://github.com/yaojianpin/acts-channel-go
