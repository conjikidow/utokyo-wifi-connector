# UTokyo Wi-Fi Connector

[![Crates.io](https://img.shields.io/crates/v/utokyo-wifi.svg?style=flat&logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/utokyo-wifi)
[![License](https://img.shields.io/crates/l/utokyo-wifi.svg?style=flat)](#license)
[![prek](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/j178/prek/master/docs/assets/badge-v0.json)](https://github.com/j178/prek)
[![CI](https://github.com/conjikidow/utokyo-wifi-connector/actions/workflows/ci.yml/badge.svg)](https://github.com/conjikidow/utokyo-wifi-connector/actions/workflows/ci.yml)

An unofficial command-line tool to manage connections to the [UTokyo Wi-Fi](https://utelecon.adm.u-tokyo.ac.jp/en/utokyo_wifi/) and eduroam networks.

`utokyo-wifi` creates NetworkManager connections for the `0000UTokyo` (UTokyo Wi-Fi) and `eduroam` networks
with your UTokyo Wi-Fi account.
The connections use PEAP with MSCHAPv2
and verify the authentication server certificate against the system CA certificates and the `u-tokyo.ac.jp` domain.

## Requirements

- Linux with NetworkManager (`nmcli`)
- A UTokyo Wi-Fi account (a user ID such as `u2499999` and its password), issued from the UTokyo Wi-Fi Account Menu

## Installation

### Prebuilt Binaries

Prebuilt binaries for x86_64 and aarch64 Linux are available from [GitHub Releases](https://github.com/conjikidow/utokyo-wifi-connector/releases).
To install the latest one into `$CARGO_HOME/bin` (`~/.cargo/bin` by default), run the installer script:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/conjikidow/utokyo-wifi-connector/releases/latest/download/utokyo-wifi-installer.sh | sh
```

To install a specific version, replace `latest/download` with `download/v<version>` in the URL, for example `download/v0.1.0`.

### Cargo

With [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), install a prebuilt binary:

```sh
cargo binstall utokyo-wifi
```

Or build it from source, which requires Rust 1.85 or later:

```sh
cargo install --locked utokyo-wifi
```

To install a specific version, append `@<version>` to the crate name, for example `utokyo-wifi@0.1.0`.

## Usage

### Set Up Connections

```sh
utokyo-wifi setup
```

`setup` asks for your user ID and password, and then for each network:

1. Creates a new connection with the account.
2. Connects to it to verify the account and the server certificate.
3. Keeps the new connection and deletes the existing connections for the same SSID, only if the connection succeeds.

If the connection fails, `setup` deletes the new connection, keeps the existing ones,
and reconnects to the network you were using.
Pressing Ctrl-C while `setup` connects does the same.
A network that is not in range is skipped, and `setup` fails if neither network is in range.
When `setup` verifies both networks, you stay connected to `eduroam`, the last one it verifies.

Use `--no-verify` to set up connections without connecting to them, for example off campus:

```sh
utokyo-wifi setup --no-verify
```

Use `--network` to choose a network (`utokyo`, `eduroam`, or `all`).
The chosen networks must be in range unless `--no-verify` is given;
otherwise `setup` fails without changing anything:

```sh
utokyo-wifi setup --network eduroam
```

To run without prompts, pass the user ID and the password on standard input:

```sh
utokyo-wifi setup --user-id u2499999 --yes --password-stdin < password-file
```

### Show Connections

```sh
utokyo-wifi status
```

`status` shows the connections for each network and whether they verify the server certificate.
Run `setup` again to replace connections that do not verify it.

### Delete Connections

```sh
utokyo-wifi delete
```

### Shell Completion

Generate a completion script for your shell (`bash`, `elvish`, `fish`, `powershell`, or `zsh`).
For example, for zsh:

```sh
utokyo-wifi completions zsh > ~/.zfunc/_utokyo-wifi
```

## License

Licensed under either of [MIT license](LICENSE-MIT) or [Apache License, Version 2.0](LICENSE-APACHE) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this software by you,
as defined in the Apache-2.0 license, shall be dually licensed as above, without any additional terms or conditions.
