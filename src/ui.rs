use crate::app::App;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::fs::read;

#[derive(Clone, Debug)]
pub enum Event {
    // Init,
    // Quit,
    // Error,
    // Closed,
    Tick,
    // Render,
    // FocusGained,
    // FocusLost,
    // Paste(String),
    Key(KeyEvent),
    // Mouse(MouseEvent),
    // Resize(u16, u16),
}

pub fn paint(f: &mut Frame, app: &mut App) {
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
    // todo - turn this into a chart
    // https://docs.rs/ratatui/latest/ratatui/widgets/struct.Chart.html
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
        (area.height - header.height - process.height) / 2,
    );
    // todo - make this into a table
    // https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html
    // also clip the query, it can be large...
    // explainer on ELU: https://nodesource.com/blog/event-loop-utilization-nodejs/
    let db_block = Block::default().title("database").borders(Borders::ALL);
    // let db_info = Paragraph::new(app.db_logs.to_owned())
    //     .scroll((0, 0))
    //     .wrap(Wrap { trim: true })
    //     .block(db_block);
    let widths = [
        Constraint::Length(5),
        Constraint::Length(10),
        Constraint::Length(45),
    ];
    let db_info = Table::new(app.db_logs.clone())
        .widths(&widths)
        // .style(Style::new().style().on_blue().underlined())
        .header(
            format_rows(
                [
                    "wait event".to_string(),
                    "state".to_string(),
                    "query".to_string(),
                ]
                .to_vec(),
            )
            .underlined(),
        )
        .block(db_block);

    // async hooks:
    // todo - turn this into a chart
    // https://docs.rs/ratatui/latest/ratatui/widgets/struct.Chart.html
    let node_async = Rect::new(
        logs.width,
        header.height + process.height + database.height,
        area.width / 2,
        (area.height - header.height - process.height) / 2,
    );
    let node_block = Block::default().title("event loop").borders(Borders::ALL);

    let elu_data = Dataset::default()
        .name("event loop utilization")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().yellow())
        .data(&app.elu_logs);

    let cpu_data = Dataset::default()
        .name("event loop utilization")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().red())
        .data(&app.cpu);

    let y_axis = Axis::default()
        .title("elu".yellow())
        .style(Style::default().white())
        .bounds([0.0, 1.2])
        .labels(vec!["0.0".into(), "0.5".into(), "1.0".into()]);

    let (start, end) = app.elu_scroll.clone();
    let x_axis = Axis::default()
        .title("time")
        .style(Style::default().white())
        .bounds([start, end]);
    // .labels(vec!["0.0".into(), "50.0".into(), "100.0".into()]);

    let node_info = Chart::new([elu_data, cpu_data].to_vec())
        .block(node_block)
        .y_axis(y_axis)
        .x_axis(x_axis);

    f.render_widget(Paragraph::new(logo).white().on_dark_gray(), header);
    f.render_widget(log_stream, logs);
    f.render_widget(app_info, process);
    f.render_stateful_widget(db_info, database, &mut app.db_state);
    f.render_widget(node_info, node_async);

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

pub fn format_rows(text_data: Vec<String>) -> Row<'static> {
    // accumulate a Vec of strings
    Row::new(text_data)
}
