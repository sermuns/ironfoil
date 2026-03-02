# ns-usbloader-rs

A tiny command-line re-implementation of [NS-USBloader](https://github.com/developersu/ns-usbloader).

![demo gif](media/demo.gif)

## Installation

When I can think of a better name than `ns-usbloader-rs` ([help me!](https://github.com/sermuns/ns-usbloader-rs/issues/1)) and have polished the code a bit more, I will publish this as a crate and start publishing binaries.

As of now, installation from source is the only options.

You have two alternatives:

1. Install using cargo

   ```sh
   cargo install --git https://github.com/sermuns/ns-usbloader-rs
   ```

2. Manually clone then install
   ```sh
   git clone https://github.com/sermuns/ns-usbloader-rs
   cd ns-usbloader-rs
   cargo install --path .
   ```

Both of these methods places `ns-usbloader-rs` in `~/.cargo/bin/`, which normally should be part of your `$PATH`.

## Usage

1. On the Nintendo Switch, open Awoo Installer and select 'Install over USB'.

2. On your PC, in a shell, run `ns-usbloader-rs <GAME_BACKUP_PATH>` where `<GAME_BACKUP_PATH>` either is the path to a directory containing the game backups or a specific game backup file.

   > For example
   >
   > ```sh
   > ns-usbloader-rs ~/Documents/NS_backups/
   > ```

3. On the Nintendo Switch, select which game(s) you want, then start installation.

4. .. profit?

Here's the output of `ns-usbloader-rs --help`:

```present cargo run -- -h
CLI alternative to NS-USBloader. Transfer data to Awoo Installer.

Usage: ns-usbloader-rs <GAME_BACKUP_PATH>

Arguments:
  <GAME_BACKUP_PATH>  

Options:
  -h, --help     Print help
  -V, --version  Print version
```
