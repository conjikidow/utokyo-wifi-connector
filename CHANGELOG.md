# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/conjikidow/utokyo-wifi-connector/releases/tag/v0.1.0) - 2026-11-22

### Added

- `setup` creates NetworkManager connections for `0000UTokyo` and `eduroam` that verify the authentication server certificate,
  and applies them only after they connect.
- `setup --no-verify` creates the connections without connecting to them.
- `status` shows the connections and whether they verify the server certificate.
- `delete` removes the connections.
- `completions` generates shell completion scripts.
