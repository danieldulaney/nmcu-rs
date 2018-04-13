# nmcu

A command-line tool for managing NodeMCU on ESP8266 chips.

## Build and run

`nmcu` is pure Rust and can be run directly with Cargo.

```
cargo run
```

Alternatively, install it with Cargo and run it as a standalone executable:

```
cargo install
nmcu
```

## Roadmap

This is still a work in progress. Here are some prospective features:

- [X] Simple two-way communication over serial
- [X] Full interactive console session
- [X] Command line arguments and subcommands
- [ ] List files
- [ ] Upload files
- [ ] Download files
- [ ] Delete files
- [ ] Persistent configuration file

