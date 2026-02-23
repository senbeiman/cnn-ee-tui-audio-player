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
            Constraint::Length(1), // フッター
            Constraint::Min(0),    // 残りスペース
        ])
        .split(f.size());

    // ヘッダー
    let status_icon = if app.is_playing() {
        if app.current_playback_repeat() {
            "🔁" // リピート再生中
        } else {
            "▶" // 通常再生中
        }
    } else {
        "⏸" // 停止中
    };
    let header_text = format!("{} {} | {}", status_icon, app.current_file_name(), app.current_directory_display());
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Green));
    f.render_widget(header, chunks[0]);

    // プレイヤー
    let progress_bar = create_progress_bar(app.progress());
    let time_text = format!("{} / {}", app.current_time_str(), app.total_time_str());
    let player_text = format!("[{}] {}", progress_bar, time_text);
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
                if app.current_playback_repeat() {
                    "🔁" // 現在の再生がリピートオン：リピートアイコン
                } else {
                    "▶ " // 現在の再生がリピートオフ：1回再生アイコン
                }
            } else {
                "  " // 再生していない
            };
            let content = format!("{}{}", play_icon, file.display_name());
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

    // フッター
    let footer_text = "[q]終了  [j/k]移動  [[/]]ページ送り  [Space]選択/再生/停止  [r]リピート再生  [Esc]上の階層";
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