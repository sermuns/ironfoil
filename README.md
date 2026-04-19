<picture>
  <source media="(prefers-color-scheme: dark)" srcset="media/banner-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="media/banner-light.svg">
  <img alt="ironfoil logo" src="media/banner-light.svg">
</picture>

A tool to transfer to [many title installers](#which-title-installers-are-supported).

Exists both as a GUI and CLI app.

<div align=center>

![gui screenshot1](media/gui-screenshot1.jpg)
![gui screenshot2](media/gui-screenshot2.jpg)
_Screenshots from `ironfoil-gui`_

</div>
<br>

<div align=center>

![cli demo](media/demo.gif)
_Demo of `ironfoil`, the core CLI application_

</div>
<br>

## Which title installers are supported?

Using the TinFoil USB protocol, I have tested:

- [Awoo Installer](https://github.com/Huntereb/Awoo-Installer)
- [CyberFoil](https://github.com/luketanti/CyberFoil)

and also:

- [Sphaira](https://github.com/ITotalJustice/sphaira), with its own protocol (make sure you enable Sphaira support when transferring!)

## Installation

> [!IMPORTANT]
> Make sure to download the right variant for you. The graphical `ironfoil-gui`, might be what you're looking for...

### Installing the GUI (`ironfoil-gui`)

Download the latest archive for your OS and extract to any location of your choice.

<div align=center>

<a href="https://github.com/sermuns/ironfoil/releases/latest/download/ironfoil-gui-x86_64-unknown-linux-gnu.tar.xz">
  <img src="media/download-linux.svg" width=200>
</a>

<a href="https://github.com/sermuns/ironfoil/releases/latest/download/ironfoil-gui-x86_64-pc-windows-msvc.zip">
  <img src="media/download-windows.svg" width=200>
</a>

<a href="https://github.com/sermuns/ironfoil/releases/latest/download/ironfoil-gui-x86_64-apple-darwin.tar.xz">
  <img src="media/download-macos.svg" width=200>
</a>

</div>

### Installing the CLI (`ironfoil`)

Alternatives:

1. Manually download [latest release](https://github.com/sermuns/ironfoil/releases/latest) and extract the tool to a location that is in your `$PATH`.

2. Use [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

   ```sh
   cargo binstall ironfoil
   ```

3. Install latest release using cargo (compile from source)

   ```sh
   cargo install ironfoil
   ```

## Usage for the CLI

### `ironfoil --help`

```present cargo run -p ironfoil -- -h
Transfer to NS title installers from the command-line

Usage: ironfoil <COMMAND>

Commands:
  usb      Transfer over USB
  network  Transfer over network
  rcm      Inject RCM payload
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### `ironfoil usb --help`

```present cargo run -p ironfoil -- usb -h
Transfer over USB

Usage: ironfoil usb [OPTIONS] <GAME_BACKUP_PATH>

Arguments:
  <GAME_BACKUP_PATH>  Path to a game backup file or directory containing game backup files

Options:
  -r, --recurse  Whether to recursively look for files (only for directories)
      --sphaira  If transferring to Sphaira homebrew menu
  -h, --help     Print help
```

### `ironfoil network --help`

```present cargo run -p ironfoil -- network -h
Transfer over network

Usage: ironfoil network [OPTIONS] <GAME_BACKUP_PATH> <TARGET_IP>

Arguments:
  <GAME_BACKUP_PATH>  Path to a game backup file or directory containing game backup files
  <TARGET_IP>         The IP address of the Nintendo Switch

Options:
  -r, --recurse  Whether to recursively look for files (only for directories)
  -h, --help     Print help
```

### `ironfoil rcm --help`

```present cargo run -p ironfoil -- rcm -h
Inject RCM payload

Usage: ironfoil rcm <PAYLOAD_PATH>

Arguments:
  <PAYLOAD_PATH>  Path to the RCM payload file

Options:
  -h, --help  Print help
```

## License

Dual-licensed under [Apache 2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT).
