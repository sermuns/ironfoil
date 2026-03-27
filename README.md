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

...

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

```present cargo run --bin ironfoil -- -h
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

## License

Dual-licensed under [Apache 2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT).
