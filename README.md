# gakumas-fast-launcher

[![Release](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml/badge.svg)](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml)

A lightweight Windows launcher for **Gakuen Idolmaster (Gakumas)** on DMM GAME PLAYER.

`gkms_fl` uses your existing DMM GAME PLAYER login session to launch the game directly, without requiring you to manually start the DMM client beforehand.

**Small binary size · Optional HTTP/SOCKS5 proxy · Written in Rust**

Release binaries are built automatically by GitHub Actions and include GitHub Artifact Attestations for build provenance verification.

## Features

* Launch Gakuen Idolmaster without manually starting DMM GAME PLAYER beforehand
* Reuse the existing DMM GAME PLAYER login session
* Optional HTTP or SOCKS5h proxy for the DMM launch API
* Automatically locate a standard DMM GAME PLAYER installation
* Show actionable error dialogs when something goes wrong
* Offer to open the official DMM GAME PLAYER when an update or re-login is required
* No background service or installer required

## Requirements

* Windows
* DMM GAME PLAYER installed
* Signed in to DMM GAME PLAYER
* Gakuen Idolmaster installed and up to date

## Usage

1. Place `gkms_fl.exe` in a writable directory.
2. Optionally, copy [`example_cfg.toml`](example_cfg.toml) to `config.toml` in the same directory.
3. Run `gkms_fl.exe`.

No configuration file is required if you are in japan.

If `config.toml` is missing, or if `dmm_proxy` / `dmm_path` are omitted, the launcher will:

* connect to DMM directly;
* look for `DMMGamePlayer.exe` in the default `%PROGRAMFILES%` installation path.

If the game cannot be launched directly, an error dialog will explain the problem. When appropriate, the dialog provides an option to open the official DMM GAME PLAYER, for example when the game needs an update or your DMM session needs to be refreshed.

## Scope and Limitations

`gkms_fl` is intentionally limited to launching an already installed and playable game(gakumas).

It does **not**:

* download or install game updates;
* update DMM GAME PLAYER;
* refresh expired DMM login sessions;
* replace the official DMM GAME PLAYER for account management;
* launch DRM-protected titles that require the official client.

When one of these operations is required, open the official DMM GAME PLAYER and handle them manully.

## Configuration

Example `config.toml`:

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

### `dmm_proxy`

Optional proxy used **only for the DMM game launch API** at:

```text
apidgp-gameplayer.games.dmm.com
```

Supported schemes:

```text
http://
socks5h://
```

Example:

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
```

Leave it empty or omit it entirely to connect directly.

### `dmm_path`

Optional path to either:

* `DMMGamePlayer.exe`, or
* the directory containing `DMMGamePlayer.exe`.

Example:

```toml
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

Leave it empty or omit it to use the default installation path under `%PROGRAMFILES%`.

## Acknowledgements

This project was heavily inspired by [DMMGamePlayerFastLauncher](https://github.com/fa0311/DMMGamePlayerFastLauncher) by yuki.

The launcher has been reimplemented in Rust with additional changes and features. The original project is distributed under the MIT License.

## License

This project is licensed under the MIT License. See [`LICENSE`](LICENSE) for details.

## Disclaimer

This is an unofficial community project and is not affiliated with, endorsed by, or maintained by DMM.com or Bandai Namco Entertainment.
