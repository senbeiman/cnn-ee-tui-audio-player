mod app;
mod files;
mod player;
mod ui;

use app::App;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

fn main() -> Result<()> {
    // コマンドライン引数から目的のディレクトリを取得
    let args: Vec<String> = std::env::args().collect();
    let dir = if args.len() > 1 {
        args[1].clone()
    } else {
        String::new() // 空文字列でデフォルトディレクトリを使用
    };

    // アプリケーション初期化
    let mut app = App::new(&dir)?;

    // ターミナル初期化
    let mut terminal = ui::setup_terminal()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen
    )?;
    terminal.hide_cursor()?;

    // メインイベントループ
    let result = run_app(&mut terminal, &mut app);

    // ターミナル復元
    ui::restore_terminal(&mut terminal)?;

    result
}

fn run_app(terminal: &mut ui::AppTerminal, app: &mut App) -> Result<()> {
    loop {
        // UI描画
        ui::draw(terminal, app)?;

        // イベント処理（100ms タイムアウト）
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(app, key)?;
            }
        }

        // 連続再生の更新処理
        app.update()?;

        // 終了条件チェック
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') => {
            app.quit();
        }
        KeyCode::Esc => {
            app.navigate_up()?;
        }
        KeyCode::Char('k') => {
            app.select_prev();
        }
        KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Char('[') => {
            app.page_up();
        }
        KeyCode::Char(']') => {
            app.page_down();
        }
        KeyCode::Char('s') => {
            app.play_selected()?;
        }
        KeyCode::Char(' ') => {
            app.toggle_pause();
        }
        KeyCode::Enter => {
            app.handle_enter_key()?;
        }
        KeyCode::Char('r') => {
            app.play_selected_repeat()?;
        }
        KeyCode::Char('c') => {
            app.play_selected_continuous()?;
        }
        _ => {}
    }

    Ok(())
}