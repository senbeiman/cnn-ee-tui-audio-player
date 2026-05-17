use anyhow::Result;
use std::path::{Path, PathBuf};

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

        Self {
            path,
            name,
            file_type,
        }
    }

    pub fn display_name(&self) -> String {
        self.name.clone()
    }

    pub fn playback_title(&self) -> String {
        strip_number_prefix(&self.name).to_string()
    }
}

fn strip_number_prefix(name: &str) -> &str {
    let digit_end = name
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .last();

    let Some(digit_end) = digit_end else {
        return name;
    };

    let rest = &name[digit_end..];
    let whitespace_end = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .last();

    if let Some(whitespace_end) = whitespace_end {
        &rest[whitespace_end..]
    } else {
        name
    }
}

pub fn scan_current_directory(current_dir: &Path, _root_dir: &Path) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // ディレクトリの存在チェック
    if !current_dir.exists() {
        anyhow::bail!("ディレクトリが見つかりません: {}", current_dir.display());
    }

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

#[cfg(test)]
mod tests {
    use super::strip_number_prefix;

    #[test]
    fn strips_ascii_number_followed_by_full_width_space() {
        assert_eq!(
            strip_number_prefix("01　Cinema Update.mp3"),
            "Cinema Update.mp3"
        );
    }

    #[test]
    fn strips_ascii_number_followed_by_half_width_space() {
        assert_eq!(
            strip_number_prefix("33 News Selection.mp3"),
            "News Selection.mp3"
        );
    }

    #[test]
    fn keeps_names_without_number_prefix() {
        assert_eq!(
            strip_number_prefix("Cinema Update.mp3"),
            "Cinema Update.mp3"
        );
    }

    #[test]
    fn keeps_leading_digits_without_separator() {
        assert_eq!(strip_number_prefix("360.mp3"), "360.mp3");
    }
}
