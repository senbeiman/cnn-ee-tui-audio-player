# 001 - ディレクトリナビゲーション機能

## 機能概要
TUI Media Playerにディレクトリ階層を移動する機能を追加し、ユーザーがディレクトリ構造を理解してファイルを選択できるようにする。

## ファイル構成

### 仕様書
- **directory-navigation.md**: 機能仕様の詳細
- **implementation-plan.md**: 段階的実装計画

## 主要な変更点

### データ構造
- `FileInfo` 構造体に `file_type` フィールド追加
- `FileType` enum（Directory/Mp3File）
- `App` 構造体に `current_directory`, `root_directory` フィールド追加

### 操作方法
- **jk**: ファイル・ディレクトリ移動（既存）
- **Space**: ディレクトリ選択で移動、MP3ファイル選択で再生
- **Esc**: 上位ディレクトリに移動（新規）

### UI変更
- ディレクトリに `📁` アイコン表示
- フッターに `[Esc]上の階層` 追加
- 現在ディレクトリパスの表示（予定）

## 実装フェーズ
1. Phase 1: データ構造拡張
2. Phase 2: ファイルスキャン変更
3. Phase 3: ナビゲーション機能
4. Phase 4: UI表示更新

## セキュリティ制限
- 起動時に指定したディレクトリ（`root_directory`）より上には移動不可
- Escキーでの移動制限を適用