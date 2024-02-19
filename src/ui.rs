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
    let logs = Rect::new(0, header.height, area.width, area.height - header.height);
    let content = Block::default().title("logs").borders(Borders::ALL);
    let log_stream = Paragraph::new(app.logs.to_owned()).block(content);

    f.render_widget(Paragraph::new(logo).white().on_dark_gray(), header);
    f.render_widget(log_stream, logs);
}
