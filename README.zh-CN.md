[English](README.md) | **简体中文** | [日本語](README.ja.md)

# gakumas-fast-launcher

[![Release](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml/badge.svg)](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml)

一款轻量级的 Windows 启动器，用于启动 DMM GAME PLAYER 上的《学园偶像大师》（Gakumas）。

`gkms_fl` 使用现有的 DMM GAME PLAYER 登录会话直接启动游戏，无需事先手动启动 DMM 客户端。

**体积小巧 · 支持可选的 HTTP/SOCKS5 代理 · 使用 Rust 编写**

发行版二进制文件由 GitHub Actions 自动构建，并附带 GitHub Artifact Attestations，可用于验证构建来源。

## 功能

* 无需事先手动启动 DMM GAME PLAYER，即可启动《学园偶像大师》
* 复用现有的 DMM GAME PLAYER 登录会话
* 可为 DMM 启动 API 配置 HTTP 或 SOCKS5h 代理
* 自动查找按默认方式安装的 DMM GAME PLAYER
* 出错时显示包含处理建议的对话框
* 需要更新或重新登录时，可选择打开官方 DMM GAME PLAYER
* 无需后台服务或安装程序

## 运行要求

* Windows
* 已安装 DMM GAME PLAYER
* 已登录 DMM GAME PLAYER
* 已安装《学园偶像大师》并更新至最新版本

## 使用方法

1. 将 `gkms_fl.exe` 放入具有写入权限的目录。
2. 如需配置，可将 [`example_cfg.toml`](example_cfg.toml) 复制到同一目录并重命名为 `config.toml`。
3. 运行 `gkms_fl.exe`。

如果你位于日本，则无需配置文件。

如果没有 `config.toml`，或者未设置 `dmm_proxy` / `dmm_path`，启动器将：

* 直接连接 DMM；
* 在 `%PROGRAMFILES%` 下的默认安装路径中查找 `DMMGamePlayer.exe`。

如果无法直接启动游戏，错误对话框会说明原因。适当情况下，对话框会提供打开官方 DMM GAME PLAYER 的选项，例如游戏需要更新或需要重新登录 DMM 时。

## 适用范围与限制

`gkms_fl` 仅用于启动已经安装且当前可正常游玩的游戏（《学园偶像大师》）。

它**不会**：

* 下载或安装游戏更新；
* 更新 DMM GAME PLAYER；
* 刷新已过期的 DMM 登录会话；
* 取代官方 DMM GAME PLAYER 管理账户；
* 启动必须使用官方客户端的 DRM 保护游戏。

需要执行上述操作时，请打开官方 DMM GAME PLAYER 手动处理。

## 配置

`config.toml` 示例：

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

### `dmm_proxy`

可选代理，仅用于访问以下 DMM 游戏启动 API：

```text
apidgp-gameplayer.games.dmm.com
```

支持以下协议：

```text
http://
socks5h://
```

示例：

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
```

留空或完全省略此项即可直接连接。

### `dmm_path`

可设置为以下任一路径：

* `DMMGamePlayer.exe` 的路径；或
* 包含 `DMMGamePlayer.exe` 的目录。

示例：

```toml
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

留空或省略此项，将使用 `%PROGRAMFILES%` 下的默认安装路径。

## 致谢

本项目深受 fa0311 的 [DMMGamePlayerFastLauncher](https://github.com/fa0311/DMMGamePlayerFastLauncher) 启发。

本启动器使用 Rust 重新实现，并加入了一些改动和功能。原项目采用 MIT 许可证发布。

## 许可证

本项目采用 MIT 许可证。详情请参阅 [`LICENSE`](LICENSE)。

## 免责声明

本项目是非官方社区项目，与 DMM.com 和 Bandai Namco Entertainment 无关，也未获得其认可或维护。
