use crate::app::App;
use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal, Frame,
};
use std::io::Stdout;

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn draw(terminal: &mut AppTerminal, app: &App) -> Result<()> {
    terminal.draw(|f| {
        draw_ui(f, app);
    })?;
    Ok(())
}

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // ヘッダー
            Constraint::Length(1), // プレイヤー
            Constraint::Length(12), // ファイルリスト（10行 + ボーダー2行）
            Constraint::Length(2), // フッター（2行）
            Constraint::Min(0),    // 残りスペース
        ])
        .split(f.size());

    // ヘッダー
    let header_text = if app.has_current_file() {
        let mode_icon = match app.playback_mode() {
            crate::player::PlaybackMode::Repeat => "🔁", // リピート再生モード
            crate::player::PlaybackMode::Continuous => "⏭️", // 連続再生モード
            crate::player::PlaybackMode::Single => "▶️", // 通常再生モード
        };
        format!("{} {} | {}", mode_icon, app.current_file_name(), app.current_directory_display())
    } else {
        format!("{}", app.current_directory_display())
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Green));
    f.render_widget(header, chunks[0]);

    // プレイヤー
    let progress_bar = create_progress_bar(app.progress());
    let time_text = format!("{} / {}", app.current_time_str(), app.total_time_str());
    let play_status_icon = if app.is_playing() {
        "▶" // 再生中
    } else {
        "⏸" // 一時停止中
    };
    let player_text = format!("{} [{}] {}", play_status_icon, progress_bar, time_text);
    let player = Paragraph::new(player_text)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(player, chunks[1]);

    // ファイルリスト（スクロール対応）
    let items: Vec<ListItem> = app.files
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(10)
        .map(|(i, file)| {
            let play_icon = if app.is_file_playing(file) {
                match app.playback_mode() {
                    crate::player::PlaybackMode::Repeat => "🔁", // リピート再生中
                    crate::player::PlaybackMode::Continuous => "⏭️", // 連続再生中
                    crate::player::PlaybackMode::Single => "▶️", // 通常再生中
                }
            } else if file.file_type == crate::files::FileType::Directory {
                "📁" // ディレクトリアイコン
            } else {
                "  " // 再生していない
            };
            let content = format!("{} {}", play_icon, file.display_name());
            let style = if i == app.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list_title = format!("ファイルリスト {}", app.current_position_info());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(list, chunks[2]);

    // フッター（2行）
    let footer_line1 = "q:終了  j:下  k:上  [:前ページ  ]:次ページ  Enter:下層  Esc:上層";
    let footer_line2 = "s:1回再生▶️  r:リピート再生🔁  c:連続再生⏭️  Space:一時停止・再開";
    let footer_text = format!("{}\n{}", footer_line1, footer_line2);
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(footer, chunks[3]);
}

fn create_progress_bar(progress: f32) -> String {
    let width = 10;
    let filled = (progress * width as f32) as usize;
    let filled_chars = "■".repeat(filled);
    let empty_chars = "░".repeat(width - filled);
    format!("{}{}", filled_chars, empty_chars)
}

pub fn setup_terminal() -> Result<AppTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut AppTerminal) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}