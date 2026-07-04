use crate::app::App;
use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
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
            Constraint::Length(5), // ファイルリスト（3行 + ボーダー2行）
            Constraint::Length(2), // フッター（2行）
            Constraint::Min(0),    // 残りスペース
        ])
        .split(f.size());

    // ヘッダー
    let header_text = if app.has_current_file() {
        format!(
            "▶️ {} | {}",
            app.current_file_name(),
            app.current_directory_display()
        )
    } else {
        format!("{}", app.current_directory_display())
    };
    let header = Paragraph::new(header_text).style(Style::default().fg(Color::Green));
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
    let player = Paragraph::new(player_text).style(Style::default().fg(Color::Cyan));
    f.render_widget(player, chunks[1]);

    // ファイルリスト（3行スロット表示）
    let entries = app.list_entries();
    let slot_indices = visible_slot_indices(app.selected, entries.len());
    let items: Vec<ListItem> = slot_indices
        .iter()
        .enumerate()
        .map(|(slot, entry_index)| {
            let Some(entry_index) = entry_index else {
                return ListItem::new(Line::from(""));
            };

            let entry = &entries[*entry_index];
            let play_icon = if app.is_list_entry_playing(entry) {
                "▶️"
            } else if entry.file_type == crate::files::FileType::Directory {
                "📁" // ディレクトリアイコン
            } else {
                "  " // 再生していない
            };
            let content = format!("{} {}", play_icon, entry.display_name);
            let style = if slot == 1 {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list_title = format!(
        "ファイルリスト {} {}",
        app.current_position_info(),
        app.speed_filter_label()
    );
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(list, chunks[2]);

    // フッター（2行）
    let footer_line1 = "q:終了  j:下へ回転  k:上へ回転  Enter:下層  Esc:上層";
    let footer_line2 = "p:再生(同名なら連続再生)  n:速度絞込  Space:一時停止・再開";
    let footer_text = format!("{}\n{}", footer_line1, footer_line2);
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::Yellow));
    f.render_widget(footer, chunks[3]);
}

fn visible_slot_indices(selected: usize, list_len: usize) -> [Option<usize>; 3] {
    match list_len {
        0 => [None, None, None],
        1 => [None, Some(0), None],
        2 => {
            let other = (selected + 1) % 2;
            [Some(other), Some(selected), Some(other)]
        }
        _ => [
            Some((selected + list_len - 1) % list_len),
            Some(selected),
            Some((selected + 1) % list_len),
        ],
    }
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
