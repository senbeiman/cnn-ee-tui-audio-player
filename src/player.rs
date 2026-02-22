use std::path::Path;
use std::time::Duration;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    current_file: Option<String>,
    is_playing: bool,
}

impl Player {
    pub fn new() -> Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;

        Ok(Self {
            _stream: stream,
            handle,
            sink: None,
            current_file: None,
            is_playing: false,
        })
    }

    pub fn play(&mut self, file_path: &Path) -> Result<()> {
        // 既存の再生を停止
        self.stop();

        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;

        // 自動リピート用にソースを無限ループ
        let repeating_source = source.repeat_infinite();

        let sink = Sink::try_new(&self.handle)?;
        sink.append(repeating_source);

        self.sink = Some(sink);
        self.current_file = Some(file_path.to_string_lossy().to_string());
        self.is_playing = true;

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if self.is_playing {
                sink.pause();
                self.is_playing = false;
            } else {
                sink.play();
                self.is_playing = true;
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self.current_file = None;
        self.is_playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_file_name(&self) -> String {
        self.current_file.as_ref()
            .map(|path| {
                Path::new(path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
            .unwrap_or_else(|| "なし".to_string())
    }

    pub fn get_position(&self) -> Duration {
        // 簡易実装：実際の位置は取得困難なため、固定値または推定値を返す
        // 本格実装では別途時間追跡機構が必要
        Duration::from_secs(0)
    }

    pub fn get_duration(&self) -> Duration {
        // 簡易実装：実際の総時間は取得困難なため、固定値を返す
        // 本格実装では音声ファイルのメタデータ解析が必要
        Duration::from_secs(300) // 5分と仮定
    }
}