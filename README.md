<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/banner-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="media/banner-light.svg">
  <img alt="ironfoil logo" src="media/banner-light.svg">
</picture>

A tiny command-line tool to transfer to [Awoo installer](https://github.com/Huntereb/Awoo-Installer).

![demo gif](media/demo.gif)

## Installation

### Binary releases

Alternatives:

1. Manually download [latest release](https://github.com/sermuns/meread/releases/latest) and extract the tool to a location that is in your `$PATH`.

2. Use [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):
   ```sh
   cargo binstall ironfoil
   ```

### From source

Alternatives:

1. Install latest release using cargo

   ```sh
   cargo install ironfoil
   ```

2. Install latest git version using cargo:

    ```sh
    cargo install --git https://github.com/sermuns/ironfoil
    ```

2. Manually clone then install
   ```sh
   git clone https://github.com/sermuns/ironfoil
   cd ironfoil
   cargo install --path cli/
   ```

Both of these methods places `ironfoil` in `~/.cargo/bin/`, which normally should be part of your `$PATH`.

## Usage

```present cargo run -- -h
Transfer to Awoo installer from the command-line

Usage: ironfoil [OPTIONS] <GAME_BACKUP_PATH> <COMMAND>

Commands:
  usb      Transfer over USB
  network  Transfer over network
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <GAME_BACKUP_PATH>  Path to a game backup file or directory containing game backup files

Options:
  -r, --recurse  Whether to recursively look for files. Only applicable when pointing to a directory
  -h, --help     Print help
  -V, --version  Print version
```

## License

Dual-licensed under [Apache 2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT).
