use crate::files::{FileInfo, FileType, scan_current_directory};
use crate::player::Player;
use anyhow::Result;

use std::path::PathBuf;

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
            PathBuf::from(format!("{}/Downloads", std::env::var("HOME").unwrap_or_default()))
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

    pub fn select_next(&mut self) {
        if !self.files.is_empty() {
            self.selected = (self.selected + 1) % self.files.len();

            // スクロール調整（10行表示の場合）
            if self.selected >= self.scroll_offset + 10 {
                self.scroll_offset = self.selected - 9;
            } else if self.selected < self.scroll_offset {
                self.scroll_offset = 0;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if !self.files.is_empty() {
            if self.selected == 0 {
                self.selected = self.files.len() - 1;
                // 最後のアイテムに移動する場合のスクロール調整
                if self.files.len() > 10 {
                    self.scroll_offset = self.files.len() - 10;
                }
            } else {
                self.selected -= 1;
            }

            // スクロール調整（10行表示の場合）
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            } else if self.selected >= self.scroll_offset + 10 {
                self.scroll_offset = self.selected - 9;
            }
        }
    }

    pub fn play_selected(&mut self) -> Result<()> {
        if let Some(file) = self.files.get(self.selected) {
            self.player.play(&file.path)?;
        }
        Ok(())
    }

    pub fn play_selected_repeat(&mut self) -> Result<()> {
        if let Some(file) = self.files.get(self.selected) {
            self.player.play_repeat(&file.path)?;
        }
        Ok(())
    }

    pub fn play_selected_continuous(&mut self) -> Result<()> {
        if let Some(file) = self.files.get(self.selected) {
            self.player.play_continuous(&file.path)?;
        }
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        self.player.toggle_pause();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn current_file_name(&self) -> String {
        if self.player.current_file_name() != "なし" {
            self.player.current_file_name()
        } else if let Some(file) = self.files.get(self.selected) {
            file.name.clone()
        } else {
            "ファイルなし".to_string()
        }
    }

    pub fn is_file_playing(&self, file: &FileInfo) -> bool {
        let current_playing = self.player.current_file_name();
        current_playing != "なし" && file.name == current_playing
    }

    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    pub fn progress(&self) -> f32 {
        // 簡易実装：進捗は固定値を返す
        let current = self.player.get_position().as_secs() as f32;
        let total = self.player.get_duration().as_secs() as f32;
        if total > 0.0 {
            current / total
        } else {
            0.0
        }
    }

    pub fn current_time_str(&self) -> String {
        let duration = self.player.get_position();
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn total_time_str(&self) -> String {
        let duration = self.player.get_duration();
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn current_position_info(&self) -> String {
        if self.files.is_empty() {
            String::new()
        } else {
            format!("({}/{})", self.selected + 1, self.files.len())
        }
    }

    pub fn current_directory_display(&self) -> String {
        // ルートディレクトリからの相対パスを表示
        if let Ok(relative_path) = self.current_directory.strip_prefix(&self.root_directory) {
            if relative_path == std::path::Path::new("") {
                "📁 ルート".to_string()
            } else {
                format!("📁 {}", relative_path.display())
            }
        } else {
            format!("📁 {}", self.current_directory.display())
        }
    }

    pub fn navigate_to_directory(&mut self, target_path: &std::path::Path) -> Result<()> {
        let new_files = scan_current_directory(target_path, &self.root_directory)?;

        self.files = new_files;
        self.current_directory = target_path.to_path_buf();
        self.selected = 0;
        self.scroll_offset = 0;

        Ok(())
    }

    pub fn navigate_up(&mut self) -> Result<()> {
        let parent = self.current_directory.parent().map(|p| p.to_path_buf());
        if let Some(parent_path) = parent {
            if parent_path >= self.root_directory {
                let old_dir_name = self.current_directory
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                self.navigate_to_directory(&parent_path)?;

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

    fn adjust_scroll(&mut self) {
        if self.selected >= self.scroll_offset + 10 {
            self.scroll_offset = self.selected - 9;
        } else if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn handle_enter_key(&mut self) -> Result<()> {
        if let Some(selected_file) = self.files.get(self.selected).cloned() {
            match selected_file.file_type {
                FileType::Directory => {
                    // ディレクトリの場合は移動
                    self.navigate_to_directory(&selected_file.path)?;
                }
                FileType::Mp3File => {
                    // MP3ファイルは何もしない（sキーで再生開始）
                }
            }
        }
        Ok(())
    }

    pub fn page_up(&mut self) {
        if self.files.is_empty() {
            return;
        }

        // 現在のscroll_offsetから10件戻る
        if self.scroll_offset >= 10 {
            self.scroll_offset -= 10;
            self.selected = self.scroll_offset;
        } else {
            // 既に最初のページにいる場合
            if self.scroll_offset == 0 && self.selected > 0 {
                // 最初のページで先頭以外にいる場合は先頭へ
                self.selected = 0;
            } else {
                // 既に先頭にいる場合は変化なし
                self.scroll_offset = 0;
                self.selected = 0;
            }
        }
    }

    pub fn page_down(&mut self) {
        if self.files.is_empty() {
            return;
        }

        // 現在のscroll_offsetから10件進む
        if self.scroll_offset + 20 < self.files.len() {
            self.scroll_offset += 10;
            self.selected = self.scroll_offset;
        } else {
            // 最後のページの場合
            if self.files.len() > 10 {
                let last_page_start = self.files.len() - 10;
                if self.scroll_offset < last_page_start {
                    // まだ最後のページでない場合は最後のページの先頭へ
                    self.scroll_offset = last_page_start;
                    self.selected = self.scroll_offset;
                } else {
                    // 既に最後のページにいる場合は最後の項目へ
                    self.selected = self.files.len() - 1;
                }
            } else {
                self.selected = self.files.len() - 1;
            }
        }
    }


    pub fn playback_mode(&self) -> crate::player::PlaybackMode {
        self.player.playback_mode()
    }

    pub fn has_current_file(&self) -> bool {
        self.player.has_current_file()
    }

    pub fn get_next_mp3_file(&self, current_index: usize) -> Option<usize> {
        for i in (current_index + 1)..self.files.len() {
            if self.files[i].file_type == crate::files::FileType::Mp3File {
                return Some(i);
            }
        }
        None
    }

    pub fn update(&mut self) -> Result<()> {
        // 連続再生モードでトラック終了をチェック
        if self.player.playback_mode() == crate::player::PlaybackMode::Continuous {
            if self.player.is_track_finished() && !self.player.is_playing() {
                // 現在再生中のファイルを特定
                let current_playing = self.player.current_file_name();
                if let Some(current_index) = self.files.iter().position(|f| f.name == current_playing) {
                    // 次のMP3ファイルを検索
                    if let Some(next_index) = self.get_next_mp3_file(current_index) {
                        // 次のトラックを再生（選択位置は維持）
                        if let Some(next_file) = self.files.get(next_index) {
                            self.player.play_continuous(&next_file.path)?;
                        }
                    } else {
                        // 最後のトラックに到達：連続再生終了
                        self.player.stop();
                    }
                }
            }
        }
        Ok(())
    }
}