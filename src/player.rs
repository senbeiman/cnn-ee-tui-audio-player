use std::path::Path;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    current_file: Option<String>,
    is_playing: bool,
    start_time: Option<Instant>,
    file_duration: Option<Duration>,
    elapsed_before_pause: Duration,
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
            start_time: None,
            file_duration: None,
            elapsed_before_pause: Duration::from_secs(0),
        })
    }

    pub fn play(&mut self, file_path: &Path) -> Result<()> {
        // 既存の再生を停止
        self.stop();

        // ファイルの長さを取得
        self.file_duration = get_mp3_duration(file_path).ok();

        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;

        // 自動リピート用にソースを無限ループ
        let repeating_source = source.repeat_infinite();

        let sink = Sink::try_new(&self.handle)?;
        sink.append(repeating_source);

        self.sink = Some(sink);
        self.current_file = Some(file_path.to_string_lossy().to_string());
        self.is_playing = true;
        self.start_time = Some(Instant::now());
        self.elapsed_before_pause = Duration::from_secs(0);

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if self.is_playing {
                // 一時停止時：現在の経過時間を保存
                if let Some(start_time) = self.start_time {
                    self.elapsed_before_pause = self.elapsed_before_pause + start_time.elapsed();
                }
                sink.pause();
                self.is_playing = false;
                self.start_time = None;
            } else {
                // 再開時：新しい開始時刻を設定
                sink.play();
                self.is_playing = true;
                self.start_time = Some(Instant::now());
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
        self.start_time = None;
        self.file_duration = None;
        self.elapsed_before_pause = Duration::from_secs(0);
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
        if let Some(duration) = self.file_duration {
            let total_elapsed = if self.is_playing {
                // 再生中：一時停止前の時間 + 現在セッションの経過時間
                if let Some(start_time) = self.start_time {
                    self.elapsed_before_pause + start_time.elapsed()
                } else {
                    self.elapsed_before_pause
                }
            } else {
                // 一時停止中：一時停止前の累積時間
                self.elapsed_before_pause
            };

            // リピート再生なので、ファイル長で割った余りを返す
            if duration.as_nanos() > 0 {
                Duration::from_nanos(
                    (total_elapsed.as_nanos() % duration.as_nanos()) as u64
                )
            } else {
                Duration::from_secs(0)
            }
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn get_duration(&self) -> Duration {
        self.file_duration.unwrap_or(Duration::from_secs(0))
    }
}

// MP3ファイルの長さを取得する関数
fn get_mp3_duration(file_path: &Path) -> Result<Duration> {
    let file = File::open(file_path)?;
    let media_source = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let format_options = FormatOptions::default();
    let metadata_options = MetadataOptions::default();

    let probed = symphonia::default::get_probe().format(
        &hint,
        media_source,
        &format_options,
        &metadata_options,
    )?;

    let mut format = probed.format;
    let track = format.tracks().iter().next()
        .ok_or_else(|| anyhow::anyhow!("No audio tracks found"))?;

    let time_base = track.codec_params.time_base;
    let frames = track.codec_params.n_frames;

    if let (Some(time_base), Some(frames)) = (time_base, frames) {
        let seconds = frames as f64 * time_base.numer as f64 / time_base.denom as f64;
        Ok(Duration::from_secs_f64(seconds))
    } else {
        // フレーム数が取得できない場合のフォールバック
        Ok(Duration::from_secs(0)) // デフォルト0分
    }
}