use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self { path, name }
    }
}

pub fn scan_mp3_files(dir: &str) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    // デフォルトディレクトリの設定
    let search_dir = if dir.is_empty() {
        format!("{}/Downloads/2603/音声DL", std::env::var("HOME").unwrap_or_default())
    } else {
        dir.to_string()
    };

    // ディレクトリの存在チェック
    let path = Path::new(&search_dir);
    if !path.exists() {
        anyhow::bail!("ディレクトリが見つかりません: {}", search_dir);
    }

    // MP3ファイルを再帰的に検索
    for entry in WalkDir::new(&search_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension.to_string_lossy().to_lowercase() == "mp3" {
                    files.push(FileInfo::new(entry.path().to_path_buf()));
                }
            }
        }
    }

    // ファイル名で自然順序ソート
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}