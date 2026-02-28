# TUI Media Player

英語学習専用のターミナル音楽プレイヤーです。

## 機能

- MP3ファイルの一覧表示・選択・再生
- リピート再生・連続再生可能
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
# デフォルトディレクトリ（~/Downloads）を使用
./target/release/tui-media-player

# 任意のディレクトリを指定
./target/release/tui-media-player /path/to/music/directory
```

## 技術仕様

- **言語**: Rust
- **TUIライブラリ**: Ratatui
- **音声ライブラリ**: Rodio
- **対応形式**: MP3のみ
- **対応OS**: macOS, Linux, Windows

## ライセンス

MIT License
