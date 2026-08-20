# gakumas-fast-launcher

[![Release](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml/badge.svg)](https://github.com/iwaDev3/gakumas-fast-launcher/actions/workflows/release.yml)

## [English](README.md) | [简体中文](README.zh-CN.md) | **日本語**

## 概要

DMM GAME PLAYER版の「学園アイドルマスター（学マス）」を起動するための、軽量なWindows向けランチャーです。

`gkms_fl`は既存のDMM GAME PLAYERのログインセッションを利用し、DMMクライアントを事前に手動で起動しなくてもゲームを直接起動します。

**軽量なバイナリ · HTTP/SOCKS5プロキシに対応（任意） · Rust製**

リリース用バイナリはGitHub Actionsで自動的にビルドされ、ビルド元を検証するためのGitHub Artifact Attestationsが付属します。

## 機能

* DMM GAME PLAYERを事前に手動で起動せずに「学園アイドルマスター」を起動
* 既存のDMM GAME PLAYERログインセッションを再利用
* DMM起動API用のHTTPまたはSOCKS5hプロキシを任意で設定可能
* 標準の場所にインストールされたDMM GAME PLAYERを自動検出
* 問題が発生した場合に、対処方法が分かるエラーダイアログを表示
* 更新や再ログインが必要な場合に、公式DMM GAME PLAYERを開く選択肢を表示
* バックグラウンドサービスやインストーラーは不要

## 動作要件

* Windows
* DMM GAME PLAYERがインストール済みであること
* DMM GAME PLAYERにログイン済みであること
* 「学園アイドルマスター」がインストール済みで、最新の状態であること

## 使い方

1. `gkms_fl.exe`を書き込み可能なディレクトリに置きます。
2. 必要に応じて、[`example_cfg.toml`](example_cfg.toml)を同じディレクトリにコピーし、`config.toml`に名前を変更します。
3. `gkms_fl.exe`を実行します。

日本国内から利用する場合、設定ファイルは不要です。

`config.toml`がない場合、または`dmm_proxy` / `dmm_path`を省略した場合、ランチャーは次のように動作します。

* DMMへ直接接続します。
* デフォルトの`%PROGRAMFILES%`インストールパスから`DMMGamePlayer.exe`を探します。

ゲームを直接起動できない場合は、エラーダイアログに原因が表示されます。ゲームの更新やDMMへの再ログインが必要な場合など、状況に応じて公式DMM GAME PLAYERを開くこともできます。

## 対象範囲と制限事項

`gkms_fl`は、インストール済みでプレイ可能なゲーム（「学園アイドルマスター」）の起動だけを目的としています。

次の操作には**対応していません**。

* ゲームアップデートのダウンロードやインストール
* DMM GAME PLAYERの更新
* 期限切れになったDMMログインセッションの更新
* 公式DMM GAME PLAYERに代わるアカウント管理
* 公式クライアントが必要なDRM保護タイトルの起動

これらの操作が必要な場合は、公式DMM GAME PLAYERを開いて手動で行ってください。

## 設定

`config.toml`の例：

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

### `dmm_proxy`

次のDMMゲーム起動APIにのみ使用する、任意のプロキシ設定です。

```text
apidgp-gameplayer.games.dmm.com
```

対応しているスキーム：

```text
http://
socks5h://
```

例：

```toml
dmm_proxy = "socks5h://127.0.0.1:20808"
```

空にするか項目自体を省略すると、DMMへ直接接続します。

### `dmm_path`

次のいずれかのパスを指定できます。

* `DMMGamePlayer.exe`
* `DMMGamePlayer.exe`があるディレクトリ

例：

```toml
dmm_path = "C:\\Program Files\\DMMGamePlayer\\DMMGamePlayer.exe"
```

空にするか項目自体を省略すると、`%PROGRAMFILES%`以下のデフォルトインストールパスを使用します。

## 謝辞

本プロジェクトは、fa0311による[DMMGamePlayerFastLauncher](https://github.com/fa0311/DMMGamePlayerFastLauncher)から多くの着想を得ています。

このランチャーはRustで再実装し、いくつかの変更と機能追加を行っています。元のプロジェクトはMIT Licenseで配布されています。

## ライセンス

本プロジェクトはMIT Licenseで公開されています。詳細は[`LICENSE`](LICENSE)をご覧ください。

## 免責事項

本プロジェクトは非公式のコミュニティプロジェクトです。DMM.comおよびBandai Namco Entertainmentとは関係がなく、両社による承認・管理も受けていません。
