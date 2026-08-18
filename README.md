# gkms_fl

A small Windows launcher for **Gakuen Idolmaster** (`gakumas`) on DMM GAME PLAYER. It uses your existing DGP login and starts the game without opening the DMM client.

**Small binary. Optional HTTP/SOCKS proxy for the DMM launch API.**

## Requirements

- Windows
- DMM GAME PLAYER installed and signed in
- Gakuen Idolmaster installed and already up to date

## Usage

1. Put `gkms_fl.exe` in a writable folder.
2. Optional: copy [`example_cfg.toml`](example_cfg.toml) to `config.toml` next to the exe.
3. Double-click `gkms_fl.exe`.

If something fails, a dialog explains what to do. Some errors offer to start official DMM GAME PLAYER. Debug builds write `debug.log`; release builds write `gkms_fl.log` next to the exe.

`config.toml` is optional. Missing file, or a valid file without `dmm_proxy` / `dmm_path`, means: connect to DMM directly, and find `DMMGamePlayer.exe` under `%PROGRAMFILES%`.

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

- `dmm_proxy`: only the launch request to `apidgp-gameplayer.games.dmm.com` (`http://` or `socks5h://`). Empty = direct.
- `dmm_path`: `DMMGamePlayer.exe` or its folder. Empty = default install path.

This launcher does not download game updates, refresh tokens, or start DRM-protected titles.

## Build

Windows GNU target (default in this repo):

```bash
cargo build --release
```

Output: `target/x86_64-pc-windows-gnu/release/gkms_fl.exe`

Host tests on Linux/WSL:

```bash
cargo test --target x86_64-unknown-linux-gnu
```

## Acknowledgements


