use crate::files::{
    build_file_list_entries, scan_current_directory, FileInfo, FileListEntry, FileType,
};
use crate::player::Player;
use anyhow::Result;

use std::path::PathBuf;

pub struct App {
    pub files: Vec<FileInfo>,
    pub selected: usize,
    pub player: Player,
    pub should_quit: bool,
    pub current_directory: PathBuf,
    pub root_directory: PathBuf,
    pub natural_speed_only: bool,
    playback_title: Option<String>,
}

impl App {
    pub fn new(dir: &str) -> Result<Self> {
        let root_dir = if dir.is_empty() {
            PathBuf::from(format!(
                "{}/Downloads",
                std::env::var("HOME").unwrap_or_default()
            ))
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
            player,
            should_quit: false,
            current_directory: current_dir,
            root_directory: root_dir,
            natural_speed_only: true,
            playback_title: None,
        })
    }

    pub fn select_next(&mut self) {
        let list_len = self.list_len();
        if list_len > 0 {
            self.selected = (self.selected + 1) % list_len;
        }
    }

    pub fn select_prev(&mut self) {
        let list_len = self.list_len();
        if list_len > 0 {
            if self.selected == 0 {
                self.selected = list_len - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn play_selected(&mut self) -> Result<()> {
        if let Some(file) = self.selected_file() {
            if file.file_type == FileType::Mp3File {
                let playback_title = file.playback_title();
                let path = file.path.clone();
                self.playback_title = Some(playback_title);
                self.player.play(&path)?;
            }
        }
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        self.player.toggle_pause();
    }

    pub fn toggle_natural_speed_filter(&mut self) {
        let selected_file_index = self
            .list_entries()
            .get(self.selected)
            .map(|entry| entry.file_index);
        self.natural_speed_only = !self.natural_speed_only;

        if let Some(file_index) = selected_file_index {
            if let Some(display_index) = self
                .display_index_for_file_index(file_index)
                .or_else(|| self.display_index_at_or_after_file_index(file_index))
            {
                self.selected = display_index;
            } else {
                self.selected = self.selected.min(self.list_len().saturating_sub(1));
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn current_file_name(&self) -> String {
        if self.player.current_file_name() != "なし" {
            self.player.current_file_name()
        } else if let Some(entry) = self.list_entries().get(self.selected) {
            entry.display_name.clone()
        } else {
            "ファイルなし".to_string()
        }
    }

    pub fn is_list_entry_playing(&self, entry: &FileListEntry) -> bool {
        let current_playing = self.player.current_file_name();
        current_playing != "なし"
            && self.files[entry.file_index..entry.file_index + entry.file_count]
                .iter()
                .any(|file| file.name == current_playing)
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
        let list_len = self.list_len();
        if list_len == 0 {
            String::new()
        } else {
            format!("({}/{})", self.selected + 1, list_len)
        }
    }

    pub fn speed_filter_label(&self) -> &'static str {
        if self.natural_speed_only {
            "ナチュラルのみ"
        } else {
            "全速度"
        }
    }

    pub fn list_entries(&self) -> Vec<FileListEntry> {
        build_file_list_entries(&self.files, self.natural_speed_only)
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

        Ok(())
    }

    pub fn navigate_up(&mut self) -> Result<()> {
        let parent = self.current_directory.parent().map(|p| p.to_path_buf());
        if let Some(parent_path) = parent {
            if parent_path >= self.root_directory {
                let old_dir_name = self
                    .current_directory
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                self.navigate_to_directory(&parent_path)?;

                // 移動元ディレクトリにカーソルを合わせる
                if let Some(pos) = self
                    .files
                    .iter()
                    .position(|f| f.file_type == FileType::Directory && f.name == old_dir_name)
                {
                    self.selected = self.display_index_for_file_index(pos).unwrap_or(0);
                }
            }
        }
        Ok(())
    }

    pub fn handle_enter_key(&mut self) -> Result<()> {
        if let Some(selected_file) = self.selected_file().cloned() {
            match selected_file.file_type {
                FileType::Directory => {
                    // ディレクトリの場合は移動
                    self.navigate_to_directory(&selected_file.path)?;
                }
                FileType::Mp3File => {
                    // MP3ファイルは何もしない（pキーで再生開始）
                }
            }
        }
        Ok(())
    }

    pub fn has_current_file(&self) -> bool {
        self.player.has_current_file()
    }

    pub fn get_next_matching_mp3_file(&self, current_index: usize) -> Option<usize> {
        let playback_title = self.playback_title.as_ref()?;

        for i in (current_index + 1)..self.files.len() {
            let file = &self.files[i];
            if file.file_type == crate::files::FileType::Mp3File {
                if file.playback_title() == *playback_title {
                    return Some(i);
                }
                return None;
            }
        }
        None
    }

    pub fn update(&mut self) -> Result<()> {
        if self.player.is_track_finished() && !self.player.is_playing() {
            // 現在再生中のファイルを特定
            let current_playing = self.player.current_file_name();
            if let Some(current_index) = self.files.iter().position(|f| f.name == current_playing) {
                if let Some(next_index) = self.get_next_matching_mp3_file(current_index) {
                    if let Some(next_file) = self.files.get(next_index) {
                        self.player.play(&next_file.path)?;
                    }
                } else {
                    self.player.stop();
                    self.playback_title = None;
                }
            } else {
                self.player.stop();
                self.playback_title = None;
            }
        }
        Ok(())
    }

    fn selected_file(&self) -> Option<&FileInfo> {
        let file_index = self.list_entries().get(self.selected)?.file_index;
        self.files.get(file_index)
    }

    fn list_len(&self) -> usize {
        self.list_entries().len()
    }

    fn display_index_for_file_index(&self, file_index: usize) -> Option<usize> {
        self.list_entries()
            .iter()
            .position(|entry| entry.file_index == file_index)
    }

    fn display_index_at_or_after_file_index(&self, file_index: usize) -> Option<usize> {
        let entries = self.list_entries();
        entries
            .iter()
            .position(|entry| entry.file_index >= file_index)
            .or_else(|| entries.len().checked_sub(1))
    }
}
