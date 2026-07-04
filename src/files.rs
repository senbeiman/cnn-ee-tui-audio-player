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

#[derive(Debug, Clone, PartialEq)]
pub struct FileListEntry {
    pub file_index: usize,
    pub file_count: usize,
    pub display_name: String,
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

pub fn build_file_list_entries(files: &[FileInfo], natural_speed_only: bool) -> Vec<FileListEntry> {
    let mut entries = Vec::new();
    let mut index = 0;

    while index < files.len() {
        let file = &files[index];

        if file.file_type == FileType::Directory {
            entries.push(FileListEntry {
                file_index: index,
                file_count: 1,
                display_name: file.display_name(),
                file_type: FileType::Directory,
            });
            index += 1;
            continue;
        }

        if natural_speed_only && !should_show_with_natural_speed_filter(files, index) {
            index += 1;
            continue;
        }

        let playback_title = file.playback_title();
        let mut file_count = 1;

        while index + file_count < files.len() {
            let next_file = &files[index + file_count];
            if next_file.file_type != FileType::Mp3File
                || next_file.playback_title() != playback_title
                || (natural_speed_only
                    && !should_show_with_natural_speed_filter(files, index + file_count))
            {
                break;
            }
            file_count += 1;
        }

        let display_name = build_mp3_display_name(files, index, file_count, &playback_title);

        entries.push(FileListEntry {
            file_index: index,
            file_count,
            display_name,
            file_type: FileType::Mp3File,
        });

        index += file_count;
    }

    entries
}

fn should_show_with_natural_speed_filter(files: &[FileInfo], index: usize) -> bool {
    let current_title = files[index].playback_title();
    let Some(current_variant) = speed_variant(&current_title) else {
        return true;
    };

    if is_natural_speed_variant(current_variant.suffix) {
        return true;
    }

    !files.iter().any(|file| {
        if file.file_type != FileType::Mp3File {
            return false;
        }

        let playback_title = file.playback_title();
        let Some(variant) = speed_variant(&playback_title) else {
            return false;
        };

        variant.base == current_variant.base && is_natural_speed_variant(variant.suffix)
    })
}

#[derive(Debug, PartialEq)]
struct SpeedVariant<'a> {
    base: &'a str,
    suffix: &'a str,
}

fn speed_variant(title: &str) -> Option<SpeedVariant<'_>> {
    let extension_start = title.rfind('.')?;
    let stem = &title[..extension_start];

    let suffix_start = stem.rfind('(').or_else(|| stem.rfind('（'))?;
    let suffix_end = stem
        .strip_suffix(')')
        .or_else(|| stem.strip_suffix('）'))
        .map(|stripped| stripped.len())?;

    if suffix_start >= suffix_end {
        return None;
    }

    Some(SpeedVariant {
        base: &stem[..suffix_start],
        suffix: &stem[suffix_start + 1..suffix_end],
    })
}

fn is_natural_speed_variant(suffix: &str) -> bool {
    suffix.contains("ナチュラル")
}

fn build_mp3_display_name(
    files: &[FileInfo],
    first_index: usize,
    file_count: usize,
    playback_title: &str,
) -> String {
    let first_number = number_prefix(&files[first_index].name);

    let Some(first_number) = first_number else {
        return playback_title.to_string();
    };

    if file_count == 1 {
        return format!("{} {}", first_number, playback_title);
    }

    let last_index = first_index + file_count - 1;
    if let Some(last_number) = number_prefix(&files[last_index].name) {
        if first_number != last_number {
            return format!("{}-{} {}", first_number, last_number, playback_title);
        }
    }

    format!("{} {}", first_number, playback_title)
}

fn strip_number_prefix(name: &str) -> &str {
    if let Some((_, title)) = split_number_prefix(name) {
        title
    } else {
        name
    }
}

fn number_prefix(name: &str) -> Option<&str> {
    split_number_prefix(name).map(|(number, _)| number)
}

fn split_number_prefix(name: &str) -> Option<(&str, &str)> {
    let digit_end = name
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .last();

    let Some(digit_end) = digit_end else {
        return None;
    };

    let rest = &name[digit_end..];
    let whitespace_end = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .last();

    if let Some(whitespace_end) = whitespace_end {
        Some((&name[..digit_end], &rest[whitespace_end..]))
    } else {
        None
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
    use super::{build_file_list_entries, strip_number_prefix, FileInfo, FileListEntry, FileType};
    use std::path::PathBuf;

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

    #[test]
    fn groups_consecutive_mp3_files_with_same_playback_title() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("01 Cinema Update.mp3"),
                name: "01 Cinema Update.mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("02 Cinema Update.mp3"),
                name: "02 Cinema Update.mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("03 News Selection.mp3"),
                name: "03 News Selection.mp3".to_string(),
                file_type: FileType::Mp3File,
            },
        ];

        assert_eq!(
            build_file_list_entries(&files, true),
            vec![
                FileListEntry {
                    file_index: 0,
                    file_count: 2,
                    display_name: "01-02 Cinema Update.mp3".to_string(),
                    file_type: FileType::Mp3File,
                },
                FileListEntry {
                    file_index: 2,
                    file_count: 1,
                    display_name: "03 News Selection.mp3".to_string(),
                    file_type: FileType::Mp3File,
                },
            ]
        );
    }

    #[test]
    fn keeps_directories_as_individual_entries() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("April"),
                name: "April".to_string(),
                file_type: FileType::Directory,
            },
            FileInfo {
                path: PathBuf::from("01 Cinema Update.mp3"),
                name: "01 Cinema Update.mp3".to_string(),
                file_type: FileType::Mp3File,
            },
        ];

        assert_eq!(
            build_file_list_entries(&files, true),
            vec![
                FileListEntry {
                    file_index: 0,
                    file_count: 1,
                    display_name: "April".to_string(),
                    file_type: FileType::Directory,
                },
                FileListEntry {
                    file_index: 1,
                    file_count: 1,
                    display_name: "01 Cinema Update.mp3".to_string(),
                    file_type: FileType::Mp3File,
                },
            ]
        );
    }

    #[test]
    fn natural_speed_filter_hides_other_variants_when_natural_exists() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("09 Training(ナチュラルスピード).mp3"),
                name: "09 Training(ナチュラルスピード).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("10 Training(ゆっくりスピード).mp3"),
                name: "10 Training(ゆっくりスピード).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("11 Training(ポーズ入り).mp3"),
                name: "11 Training(ポーズ入り).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("12 Story.mp3"),
                name: "12 Story.mp3".to_string(),
                file_type: FileType::Mp3File,
            },
        ];

        assert_eq!(
            build_file_list_entries(&files, true),
            vec![
                FileListEntry {
                    file_index: 0,
                    file_count: 1,
                    display_name: "09 Training(ナチュラルスピード).mp3".to_string(),
                    file_type: FileType::Mp3File,
                },
                FileListEntry {
                    file_index: 3,
                    file_count: 1,
                    display_name: "12 Story.mp3".to_string(),
                    file_type: FileType::Mp3File,
                },
            ]
        );
    }

    #[test]
    fn natural_speed_filter_keeps_other_variants_when_no_natural_exists() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("50 News(1.5倍速).mp3"),
                name: "50 News(1.5倍速).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("51 News(1.5倍速).mp3"),
                name: "51 News(1.5倍速).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
        ];

        assert_eq!(
            build_file_list_entries(&files, true),
            vec![FileListEntry {
                file_index: 0,
                file_count: 2,
                display_name: "50-51 News(1.5倍速).mp3".to_string(),
                file_type: FileType::Mp3File,
            }]
        );
    }

    #[test]
    fn natural_speed_filter_can_be_disabled() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("09 Training(ナチュラルスピード).mp3"),
                name: "09 Training(ナチュラルスピード).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
            FileInfo {
                path: PathBuf::from("10 Training(ゆっくりスピード).mp3"),
                name: "10 Training(ゆっくりスピード).mp3".to_string(),
                file_type: FileType::Mp3File,
            },
        ];

        assert_eq!(build_file_list_entries(&files, false).len(), 2);
    }
}
