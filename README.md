# DATEX CLI

## Usage

### Running the REPL
```shell
datex
```
Alternatively, you can also use the `repl` subcommand:
```shell
datex repl
```

To show debug information, run the `repl` subcommand with the `--verbose` or `-v` flag:
```shell
datex repl -v
```

### Running a DATEX file
```shell
datex run path/to/file.dx
```

## Development
### Running the REPL
```shell
cargo run
```

### Running the Workbench
```shell
cargo run workbench
```

### Building for Release
```shell
cargo build --release
./target/release/datex_cli
```