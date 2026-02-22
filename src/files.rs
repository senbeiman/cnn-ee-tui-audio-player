use std::path::{Path, PathBuf};
use anyhow::Result;

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

