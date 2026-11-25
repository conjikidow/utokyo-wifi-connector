# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/conjikidow/utokyo-wifi-connector/compare/v0.1.2...v0.1.3) - 2026-11-25

### Documentation

- add a Japanese README ([#28](https://github.com/conjikidow/utokyo-wifi-connector/pull/28))
- state the target platform and clarify the README ([#27](https://github.com/conjikidow/utokyo-wifi-connector/pull/27))
- link the official UTokyo Wi-Fi page ([#23](https://github.com/conjikidow/utokyo-wifi-connector/pull/23))

## [0.1.2](https://github.com/conjikidow/utokyo-wifi-connector/compare/v0.1.1...v0.1.2) - 2026-11-25

### Documentation

- state the minimum supported Rust version ([#20](https://github.com/conjikidow/utokyo-wifi-connector/pull/20))

### Other

- *(cargo)* declare and test the minimum supported Rust version ([#17](https://github.com/conjikidow/utokyo-wifi-connector/pull/17))

## [0.1.1](https://github.com/conjikidow/utokyo-wifi-connector/compare/v0.1.0...v0.1.1) - 2026-11-22

### Documentation

- explain how to install a specific version ([#9](https://github.com/conjikidow/utokyo-wifi-connector/pull/9))
- add README badges and list the MIT license first ([#7](https://github.com/conjikidow/utokyo-wifi-connector/pull/7))

## [0.1.0](https://github.com/conjikidow/utokyo-wifi-connector/releases/tag/v0.1.0) - 2026-11-22

### Added

- `setup` creates NetworkManager connections for `0000UTokyo` and `eduroam` that verify the authentication server certificate,
  and applies them only after they connect.
- `setup --no-verify` creates the connections without connecting to them.
- `status` shows the connections and whether they verify the server certificate.
- `delete` removes the connections.
- `completions` generates shell completion scripts.
