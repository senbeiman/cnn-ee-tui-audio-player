# TUI Media Player 技術仕様書

## 1. 技術スタック

### 依存関係
```toml
[dependencies]
ratatui = "0.24"      # TUI
crossterm = "0.27"    # クロスプラットフォーム入力
rodio = "0.17"        # 音声再生
walkdir = "2.4"       # ディレクトリ走査
```

### アーキテクチャ
```
main.rs
├── app.rs           # アプリ状態管理
├── ui.rs            # 画面描画
├── player.rs        # 音声再生
└── files.rs         # ファイル管理
```

## 2. コアデータ構造

```rust
// src/app.rs
struct App {
    files: Vec<FileInfo>,
    selected: usize,
    player: Player,
    should_quit: bool,
}

// src/files.rs
struct FileInfo {
    path: PathBuf,
    name: String,
}

// src/player.rs
struct Player {
    sink: Option<Sink>,
    current_time: Duration,
    total_time: Duration,
    is_playing: bool,
}
```

## 3. 主要関数

### main.rs
```rust
fn main() -> Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_default();
    let mut app = App::new(&dir)?;

    loop {
        ui::draw(&app)?;

        if let Ok(event) = event::read() {
            match event {
                KeyEvent { code: KeyCode::Char('q'), .. } => break,
                KeyEvent { code: KeyCode::Up, .. } => app.select_prev(),
                KeyEvent { code: KeyCode::Down, .. } => app.select_next(),
                KeyEvent { code: KeyCode::Enter, .. } => app.play_selected()?,
                KeyEvent { code: KeyCode::Char(' '), .. } => app.toggle_pause(),
                _ => {}
            }
        }

        app.update_player_state();
    }

    Ok(())
}
```

### files.rs
```rust
pub fn scan_mp3_files(dir: &str) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.path().extension() == Some("mp3".as_ref()) {
            files.push(FileInfo {
                path: entry.path().to_path_buf(),
                name: entry.file_name().to_string_lossy().to_string(),
            });
        }
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}
```

### player.rs
```rust
pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}

impl Player {
    pub fn play(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;

        self.sink = Some(Sink::try_new(&self.handle)?);
        self.sink.as_ref().unwrap().append(source);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }
}
```

### ui.rs
```rust
pub fn draw(app: &App) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // ヘッダー
                Constraint::Length(1), // プレイヤー
                Constraint::Min(0),    // ファイルリスト
                Constraint::Length(1), // フッター
            ])
            .split(f.size());

        // ヘッダー
        let header = Paragraph::new(format!("♪ {}", app.current_file_name()))
            .style(Style::default().fg(Color::Green));
        f.render_widget(header, chunks[0]);

        // プレイヤー
        let progress = format!("[{}] {:02}:{:02} / {:02}:{:02}",
            progress_bar(app.progress()),
            app.current_time.as_secs() / 60,
            app.current_time.as_secs() % 60,
            app.total_time.as_secs() / 60,
            app.total_time.as_secs() % 60
        );
        let player = Paragraph::new(progress);
        f.render_widget(player, chunks[1]);

        // ファイルリスト
        let items: Vec<ListItem> = app.files.iter().enumerate()
            .map(|(i, file)| {
                let prefix = if i == app.selected { "> " } else { "  " };
                ListItem::new(format!("{}{}", prefix, file.name))
                    .style(if i == app.selected {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    })
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, chunks[2]);

        // フッター
        let footer = Paragraph::new("[q]終了 [↑↓]選択 [Enter]再生 [Space]停止");
        f.render_widget(footer, chunks[3]);
    })?;

    Ok(())
}

fn progress_bar(progress: f32) -> String {
    let width = 10;
    let filled = (progress * width as f32) as usize;
    "■".repeat(filled) + &"░".repeat(width - filled)
}
```

## 4. ビルド・実行

```bash
# ビルド
cargo build --release

# 実行
./target/release/tui-media-player ~/Downloads/2603/音声DL/
```

## 5. ファイル構成

```
src/
├── main.rs          # 100行程度
├── app.rs           # 80行程度
├── ui.rs            # 120行程度
├── player.rs        # 60行程度
└── files.rs         # 40行程度

合計: 約400行
```

---

**作成日**: 2026-02-22
**バージョン**: 1.0
**総実装規模**: 約400行のRustコード