use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let logo = r"                               ___                            _
      /\  /\___ _   _  ___    / __\___  _ __ ___  _ __  _   _| |_ ___ _ __
     / /_/ / _ \ | | |/ _ \  / /  / _ \| '_ ` _ \| '_ \| | | | __/ _ \ '__|
    / __  /  __/ |_| | (_) |/ /__| (_) | | | | | | |_) | |_| | ||  __/ |
    \/ /_/ \___|\__, |\___(_)____/\___/|_| |_| |_| .__/ \__,_|\__\___|_|
                |___/                            |_|                       ";

    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    // app title:
    let area = f.size();
    let header = Rect::new(area.x, area.y, area.width, area.height / 6);

    // log stream:
    let logs = Rect::new(
        0,
        header.height,
        area.width / 2,
        area.height - header.height,
    );
    let log_block = Block::default().title("logs").borders(Borders::ALL);

    static LOG_BUFFER: u16 = 10;
    let vert_scroll = if app.scroll_pos > LOG_BUFFER {
        app.scroll_pos - 10
    } else {
        app.scroll_pos
    };

    let log_stream = Paragraph::new(app.logs.to_owned())
        .scroll((vert_scroll, 0))
        .wrap(Wrap { trim: true })
        .block(log_block);

    // process info:
    let process = Rect::new(
        logs.width,
        header.height,
        area.width / 2,
        (area.height - header.height) / 3,
    );
    let info_block = Block::default().title("process").borders(Borders::ALL);
    let app_info = Paragraph::new(app.process.to_owned()).block(info_block);

    // database info:
    let database = Rect::new(
        logs.width,
        header.height + process.height,
        area.width / 2,
        area.height - header.height - process.height,
    );
    let db_block = Block::default().title("database").borders(Borders::ALL);
    let db_info = Paragraph::new(app.db_logs.to_owned()).block(db_block);

    f.render_widget(Paragraph::new(logo).white().on_dark_gray(), header);
    f.render_widget(log_stream, logs);
    f.render_widget(app_info, process);
    f.render_widget(db_info, database);

    // scrollbar for logs:

    let mut scrollbar_state =
        ScrollbarState::new(app.scroll_pos as usize).position(vert_scroll as usize);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        logs.inner(&Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
