# UTokyo Wi-Fi Connector

[![Crates.io](https://img.shields.io/crates/v/utokyo-wifi.svg?style=flat&logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/utokyo-wifi)
[![License](https://img.shields.io/crates/l/utokyo-wifi.svg?style=flat)](#ライセンス)
[![prek](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/j178/prek/master/docs/assets/badge-v0.json)](https://github.com/j178/prek)
[![CI](https://github.com/conjikidow/utokyo-wifi-connector/actions/workflows/ci.yml/badge.svg)](https://github.com/conjikidow/utokyo-wifi-connector/actions/workflows/ci.yml)

[English](README.md) | **日本語**

東京大学が提供する [UTokyo Wi-Fi](https://utelecon.adm.u-tokyo.ac.jp/utokyo_wifi/) と eduroam への接続設定を行う，Linux 向けの非公式コマンドラインツールです。

`utokyo-wifi` は，UTokyo Wi-Fi アカウントを使い，`0000UTokyo`（UTokyo Wi-Fi）と `eduroam` の NetworkManager 接続プロファイルを作成します。
接続時に認証サーバの証明書を検証し，偽のアクセスポイントにパスワードが漏れるのを防ぎます。

## 必要なもの

- NetworkManager（`nmcli`）が使える Linux
- UTokyo Wi-Fi アカウントメニューにて発行された，UTokyo Wi-Fi アカウント（`u2499999` のようなユーザー ID とそのパスワード）

## インストール

### ビルド済みバイナリ

Linux（x86_64，aarch64）向けのビルド済みバイナリを [GitHub Releases](https://github.com/conjikidow/utokyo-wifi-connector/releases) で配布しています。
最新版を `$CARGO_HOME/bin`（既定では `~/.cargo/bin`）にインストールするには，インストーラスクリプトを実行します。

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/conjikidow/utokyo-wifi-connector/releases/latest/download/utokyo-wifi-installer.sh | sh
```

特定のバージョンをインストールするには，URL の `latest/download` を `download/v<version>` に置き換えます（例：`download/v0.1.2`）。

### Cargo

[cargo-binstall](https://github.com/cargo-bins/cargo-binstall) を使うと，ビルド済みバイナリをインストールできます。

```sh
cargo binstall utokyo-wifi
```

Rust 1.85 以降を用いてソースからビルドすることもできます。

```sh
cargo install --locked utokyo-wifi
```

特定のバージョンをインストールするには，クレート名に `@<version>` を付けます（例：`utokyo-wifi@0.1.2`）。

## 使い方

### 接続プロファイルの設定

```sh
utokyo-wifi setup
```

`setup` はユーザー ID とパスワードの入力後，ネットワークごとに次の処理を行います。

1. 入力したアカウントを使って，新しい接続プロファイルを作成します。
2. そのプロファイルでネットワークに接続し，アカウントとサーバ証明書を検証します。
3. 接続に成功した場合に限り，そのプロファイルを残し，同じ SSID の既存プロファイルを削除します。

接続に失敗した場合，`setup` は作成したプロファイルを削除して既存プロファイルを残し，それまで使っていたネットワークに接続し直します。
`setup` が接続を試している間に Ctrl-C を押した場合も同様です。
圏外のネットワークはスキップし，どちらのネットワークも圏外なら `setup` は失敗します。
両方のネットワークで接続の検証に成功した場合は，最後に検証した `eduroam` に接続したままになります。

キャンパスの外などでネットワークに接続せずに接続プロファイルを設定するには，`--no-verify` を使います。

```sh
utokyo-wifi setup --no-verify
```

接続先ネットワークは `--network` で選べます（`utokyo`，`eduroam`，`all`）。
`--no-verify` を付けない場合，`--network` で選んだネットワークはすべて圏内にある必要があります。
1つでも圏外なら，`setup` は何も変更せずに失敗します。

```sh
utokyo-wifi setup --network eduroam
```

プロンプトを表示せずに実行するには，ユーザー ID を `--user-id` で指定し，`--yes` を付けます。
パスワードは，`--password-stdin` を付けて標準入力から渡します。

```sh
utokyo-wifi setup --user-id u2499999 --yes --password-stdin < password-file
```

### 接続プロファイルの表示

```sh
utokyo-wifi status
```

`status` は，ネットワークごとの接続プロファイルと，それぞれがサーバ証明書を検証するかどうかを表示します。
サーバ証明書を検証しない接続プロファイルは，`setup` をもう一度実行して置き換えてください。

### 接続プロファイルの削除

```sh
utokyo-wifi delete
```

### シェル補完

シェル（`bash`，`elvish`，`fish`，`powershell`，`zsh`）の補完スクリプトを生成します。
たとえば zsh の場合，次を実行します。

```sh
utokyo-wifi completions zsh > ~/.zfunc/_utokyo-wifi
```

## ライセンス

本ソフトウェアは [MIT ライセンス](LICENSE-MIT) と [Apache License, Version 2.0](LICENSE-APACHE) のデュアルライセンスです。
どちらか一方を選んで利用できます。

本ソフトウェアへの Apache-2.0 ライセンスで定められた貢献は，あなたが別の意思を明示しない限り，追加の条項や条件なしに上記のデュアルライセンスで提供されるものとします。
