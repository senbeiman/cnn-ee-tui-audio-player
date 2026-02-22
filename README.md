# TUI Media Player

英語学習専用のターミナル音楽プレイヤーです。

## 機能

- MP3ファイルの一覧表示・選択・再生
- 自動リピート再生（デフォルトON）
- シンプルなキーボード操作
- 日本語ファイル名対応
- 進捗表示

## 使い方

### インストール

```bash
# Rustをインストール済みの場合
cargo build --release
```

### 実行

```bash
# デフォルトディレクトリ（~/Downloads/2603/音声DL/）を使用
./target/release/tui-media-player

# 任意のディレクトリを指定
./target/release/tui-media-player /path/to/music/directory
```

### キーバインド

| キー | 動作 |
|------|------|
| `↑` | 前のファイル |
| `↓` | 次のファイル |
| `Enter` | 選択ファイル再生 |
| `Space` | 再生/一時停止 |
| `q` | 終了 |

## 技術仕様

- **言語**: Rust
- **TUIライブラリ**: Ratatui
- **音声ライブラリ**: Rodio
- **対応形式**: MP3のみ
- **対応OS**: macOS, Linux, Windows

## ライセンス

MIT License