# CNN EE TUI Audio Player

[CNN English Express](https://ee.asahipress.com/latest-issue/) の音声ファイルをCLIで再生するための専用TUIプレイヤーです。

CNN EEの音声ファイル名に付く先頭番号を除いて同名判定し、同じ記事・セクションの音声だけを続けて再生します。

## 機能

- CNN EEのMP3ファイル一覧表示・選択・再生
- 先頭番号を除いた同名トラックのグループ表示・連続再生
- ナチュラルスピード音声の絞り込み表示
- シンプルなキーボード操作
- 日本語ファイル名対応
- 進捗表示

## 使い方

### インストール・更新

Rustをインストール済みの環境では、リポジトリのルートで次を実行します。

```bash
cargo install --path . --force
```

このコマンドは現在のソースコードをビルドし、`play-cnnee` コマンドとして `~/.cargo/bin` にインストールします。すでにインストール済みの場合も最新版で上書きします。

`play-cnnee` が見つからない場合は、`~/.cargo/bin` にPATHが通っているか確認してください。

```bash
echo $PATH
which play-cnnee
```

zshの場合は `~/.zshrc` に次の設定があれば使えます。

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### 起動

```bash
# デフォルトディレクトリ（~/Downloads）を使用
play-cnnee

# 任意のディレクトリを指定
play-cnnee /path/to/music/directory
```

### キー操作

```text
j / k  選択を下/上へ移動
[ / ]  前/次ページへ移動
Enter  ディレクトリに入る
Esc    上のディレクトリに戻る
p      再生（同名なら連続再生）
n      ナチュラルスピード絞り込みの切り替え
Space  一時停止・再開
q      終了
```

## 技術仕様

- **言語**: Rust
- **TUIライブラリ**: Ratatui
- **音声ライブラリ**: Rodio
- **対応形式**: MP3のみ
- **対応OS**: macOS, Linux, Windows

## ライセンス

MIT License
