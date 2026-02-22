use crate::files::{FileInfo, scan_mp3_files};
use crate::player::Player;
use anyhow::Result;

pub struct App {
    pub files: Vec<FileInfo>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub player: Player,
    pub should_quit: bool,
}

impl App {
    pub fn new(dir: &str) -> Result<Self> {
        let files = scan_mp3_files(dir)?;
        let player = Player::new()?;

        if files.is_empty() {
            anyhow::bail!("MP3ファイルが見つかりません");
        }

        Ok(Self {
            files,
            selected: 0,
            scroll_offset: 0,
            player,
            should_quit: false,
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

    pub fn handle_space_key(&mut self) -> Result<()> {
        if let Some(selected_file) = self.files.get(self.selected) {
            let selected_path = selected_file.path.to_string_lossy().to_string();
            let current_playing = self.player.current_file_name();

            // 選択中のファイルが再生中かどうかをチェック
            if current_playing != "なし" && selected_file.name == current_playing {
                // 再生中のファイルが選択されている場合は一時停止/再開
                self.toggle_pause();
            } else {
                // 違うファイルが選択されている場合は新しいファイルを再生
                self.play_selected()?;
            }
        }
        Ok(())
    }
}