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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackMode {
    Single,      // 通常再生（1回）
    Repeat,      // リピート再生
    Continuous,  // 連続再生
}

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    current_file: Option<String>,
    is_playing: bool,
    start_time: Option<Instant>,
    file_duration: Option<Duration>,
    elapsed_before_pause: Duration,
    playback_mode: PlaybackMode, // 現在の再生モード
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
            playback_mode: PlaybackMode::Single,
        })
    }

    pub fn play(&mut self, file_path: &Path) -> Result<()> {
        self.play_with_mode(file_path, PlaybackMode::Single)
    }

    pub fn play_repeat(&mut self, file_path: &Path) -> Result<()> {
        self.play_with_mode(file_path, PlaybackMode::Repeat)
    }

    pub fn play_continuous(&mut self, file_path: &Path) -> Result<()> {
        self.play_with_mode(file_path, PlaybackMode::Continuous)
    }

    fn play_with_mode(&mut self, file_path: &Path, mode: PlaybackMode) -> Result<()> {
        // 既存の再生を停止
        self.stop();

        // ファイルの長さを取得
        self.file_duration = get_mp3_duration(file_path).ok();

        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;

        let sink = Sink::try_new(&self.handle)?;

        match mode {
            PlaybackMode::Repeat => {
                // リピート再生：無限ループ
                sink.append(source.repeat_infinite());
            }
            PlaybackMode::Single | PlaybackMode::Continuous => {
                // 通常再生または連続再生：1回だけ再生
                sink.append(source);
            }
        }

        self.sink = Some(sink);
        self.current_file = Some(file_path.to_string_lossy().to_string());
        self.is_playing = true;
        self.start_time = Some(Instant::now());
        self.elapsed_before_pause = Duration::from_secs(0);
        self.playback_mode = mode;

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
        self.playback_mode = PlaybackMode::Single;
    }

    pub fn is_playing(&self) -> bool {
        // リピート以外で曲が終了している場合は停止状態とする
        if self.playback_mode != PlaybackMode::Repeat {
            if let Some(duration) = self.file_duration {
                let total_elapsed = if self.is_playing {
                    if let Some(start_time) = self.start_time {
                        self.elapsed_before_pause + start_time.elapsed()
                    } else {
                        self.elapsed_before_pause
                    }
                } else {
                    self.elapsed_before_pause
                };

                if total_elapsed >= duration {
                    return false;
                }
            }
        }

        self.is_playing
    }

    pub fn current_file_name(&self) -> String {
        // Single再生で終了している場合は"なし"を返す（Continuousは除く）
        if self.playback_mode == PlaybackMode::Single {
            if let Some(duration) = self.file_duration {
                let total_elapsed = if self.is_playing {
                    if let Some(start_time) = self.start_time {
                        self.elapsed_before_pause + start_time.elapsed()
                    } else {
                        self.elapsed_before_pause
                    }
                } else {
                    self.elapsed_before_pause
                };

                if total_elapsed >= duration {
                    return "なし".to_string();
                }
            }
        }

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

            if duration.as_nanos() > 0 {
                if self.playback_mode == PlaybackMode::Repeat {
                    // リピート再生：ファイル長で割った余りを返す（無限リピート）
                    Duration::from_nanos(
                        (total_elapsed.as_nanos() % duration.as_nanos()) as u64
                    )
                } else {
                    // 通常再生または連続再生：ファイル長を超えたら終了位置で固定
                    if total_elapsed >= duration {
                        duration
                    } else {
                        total_elapsed
                    }
                }
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

    pub fn playback_mode(&self) -> PlaybackMode {
        self.playback_mode
    }


    pub fn is_track_finished(&self) -> bool {
        if let Some(duration) = self.file_duration {
            let total_elapsed = if self.is_playing {
                if let Some(start_time) = self.start_time {
                    self.elapsed_before_pause + start_time.elapsed()
                } else {
                    self.elapsed_before_pause
                }
            } else {
                self.elapsed_before_pause
            };

            total_elapsed >= duration
        } else {
            false
        }
    }

    pub fn has_current_file(&self) -> bool {
        self.current_file.is_some()
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

    let format = probed.format;
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