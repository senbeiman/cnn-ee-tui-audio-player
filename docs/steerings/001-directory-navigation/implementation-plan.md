# ディレクトリナビゲーション機能 実装計画

## 実装フェーズ

### Phase 1: データ構造の拡張
1. **FileTypeとFileInfo構造体の拡張**
   - `src/files.rs` の `FileInfo` に `file_type` フィールド追加
   - `FileType` enum の実装

2. **App構造体の拡張**
   - `src/app.rs` の `App` に `current_directory`, `root_directory` フィールド追加
   - 初期化ロジックの更新

### Phase 2: ファイルスキャン機能の変更
1. **scan_current_directory関数の実装**
   - `src/files.rs` の `scan_mp3_files` を置き換え
   - 再帰的スキャンから単一階層スキャンに変更
   - ディレクトリとファイルの両方を検出

2. **Escキーによる上位移動**
   - キーハンドリングでEscキー処理追加

### Phase 3: ナビゲーション機能の実装
1. **ディレクトリ移動メソッド**
   - `src/app.rs` に `navigate_to_directory()` メソッド追加
   - `navigate_up()` メソッド追加

2. **Space キーハンドリングの拡張**
   - `handle_space_key()` でファイルタイプ判定を追加
   - ディレクトリ選択時の移動処理

### Phase 4: UI表示の更新
1. **ファイルリストの表示改善**
   - `src/ui.rs` でディレクトリプレフィックス `[DIR]` の表示
   - ファイルタイプ別のスタイリング

2. **パス表示の追加**
   - ヘッダーまたはタイトルバーに現在ディレクトリパス表示

## 実装の詳細

### Phase 1: データ構造の拡張

#### files.rs の変更
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Directory,
    Mp3File,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
}

impl FileInfo {
    pub fn new(path: PathBuf, file_type: FileType) -> Self {
        let name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self { path, name, file_type }
    }

    pub fn display_name(&self) -> String {
        match self.file_type {
            FileType::Directory => format!("📁 {}", self.name),
            FileType::Mp3File => self.name.clone(),
        }
    }
}
```

#### app.rs の変更
```rust
pub struct App {
    pub files: Vec<FileInfo>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub player: Player,
    pub should_quit: bool,
    pub current_directory: PathBuf,
    pub root_directory: PathBuf,
}

impl App {
    pub fn new(dir: &str) -> Result<Self> {
        let root_dir = if dir.is_empty() {
            PathBuf::from(format!("{}/Downloads/2603/音声DL", std::env::var("HOME").unwrap_or_default()))
        } else {
            PathBuf::from(dir)
        };

        let current_dir = root_dir.clone();
        let files = scan_current_directory(&current_dir, &root_dir)?;
        let player = Player::new()?;

        if files.is_empty() {
            anyhow::bail!("ファイルが見つかりません");
        }

        Ok(Self {
            files,
            selected: 0,
            scroll_offset: 0,
            player,
            should_quit: false,
            current_directory: current_dir,
            root_directory: root_dir,
        })
    }
}
```

### Phase 2: ファイルスキャン機能

#### scan_current_directory関数
```rust
pub fn scan_current_directory(current_dir: &Path, root_dir: &Path) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // 現在ディレクトリの内容をスキャン
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.push(FileInfo::new(path, FileType::Directory));
        } else if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "mp3" {
                    files.push(FileInfo::new(path, FileType::Mp3File));
                }
            }
        }
    }

    // ソート: ディレクトリ → ファイル の順番
    files.sort_by(|a, b| {
        use FileType::*;
        match (&a.file_type, &b.file_type) {
            (Directory, Directory) => a.name.cmp(&b.name),
            (Directory, Mp3File) => std::cmp::Ordering::Less,
            (Mp3File, Directory) => std::cmp::Ordering::Greater,
            (Mp3File, Mp3File) => a.name.cmp(&b.name),
        }
    });

    Ok(files)
}
```

### Phase 3: ナビゲーション機能

#### ディレクトリ移動メソッド
```rust
impl App {
    pub fn navigate_to_directory(&mut self, target_path: &Path) -> Result<()> {
        let new_files = scan_current_directory(target_path, &self.root_directory)?;

        self.files = new_files;
        self.current_directory = target_path.to_path_buf();
        self.selected = 0;
        self.scroll_offset = 0;

        Ok(())
    }

    pub fn navigate_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_directory.parent() {
            if parent >= &self.root_directory {
                let old_dir_name = self.current_directory
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                self.navigate_to_directory(parent)?;

                // 移動元ディレクトリにカーソルを合わせる
                if let Some(pos) = self.files.iter().position(|f|
                    f.file_type == FileType::Directory && f.name == old_dir_name
                ) {
                    self.selected = pos;
                    self.adjust_scroll();
                }
            }
        }
        Ok(())
    }
}
```

### Phase 4: UI更新

#### ファイルリスト表示の変更
```rust
// ui.rs の変更
let content = format!("{}{}", prefix, file.display_name());
```

## マイルストーン

### Milestone 1 (Phase 1-2完了)
- データ構造変更
- 単一階層ファイルスキャン
- 基本的な表示（ディレクトリプレフィックス付き）

### Milestone 2 (Phase 3完了)
- ディレクトリナビゲーション機能
- Space キーでディレクトリ移動
- ".." による上位移動

### Milestone 3 (Phase 4完了)
- UI改善
- パス表示
- 最終テスト

## リスク評価

### 高リスク
- 既存のファイル再生機能への影響
- 複雑なディレクトリ権限に対するエラーハンドリング

### 中リスク
- UI表示の複雑化によるパフォーマンス影響
- キーバインド競合

### 低リスク
- ファイルソート順序の調整