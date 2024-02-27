use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind};
use tokio::{sync::mpsc, task::JoinHandle};
use tui_input::Input;

use crate::ui::Event;
use futures::{FutureExt, StreamExt};

pub enum InputMode {
    Normal,
    Editing, // should change this to "db" or something
}

pub struct App {
    /// Current value of the input box
    pub input: Input,
    /// Current input mode
    pub input_mode: InputMode,
    /// command to run
    pub cmd: String,
    /// logs from the child process
    pub logs: String,
    pub scroll_pos: u16,
    /// process info:
    pub process: String,
    /// database info:
    pub db_type: String,
    pub db_logs: String,
    pub task: Option<JoinHandle<()>>,
    pub should_exit: bool,
    pub rx: mpsc::UnboundedReceiver<Event>,
    pub tx: mpsc::UnboundedSender<Event>,
}

impl App {
    pub fn new(cmd: &str, db: &str) -> App {
        let (tx, rx) = mpsc::unbounded_channel();
        App {
            input: Input::default(),
            input_mode: InputMode::Normal,
            cmd: String::from(cmd),
            logs: String::new(),
            scroll_pos: 0,
            process: String::from("fetching info..."),
            db_type: String::from(db),
            db_logs: String::from("searching for connections..."),
            task: None,
            should_exit: false,
            tx,
            rx,
        }
    }

    pub fn start(&mut self) {
        // Async stuff:
        // todo - be smarter about these refresh intervals!
        let tick_delay = std::time::Duration::from_secs_f64(1.0);
        let render_delay = std::time::Duration::from_secs_f64(1.0);
        let _event_tx = self.tx.clone();

        self.task = Some(tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            let mut reader = crossterm::event::EventStream::new();

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();

                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    _ => {
                                        _event_tx.send(Event::Tick);
                                        ()
                                    }
                                }
                            },
                            None => (),
                            Some(Error) => ()
                        }
                    }
                }
            }
        }));
    }

    pub fn update_logs(&mut self, log: &str) {
        self.logs.push_str(log);
        self.logs.push_str("\n");
        self.add_scroll_count()
    }

    pub fn update_process(&mut self, info: &str) {
        self.process = String::from(info)
    }

    pub fn add_scroll_count(&mut self) {
        self.scroll_pos += 1
    }

    pub fn update_db_logs(&mut self, log: &str) {
        self.db_logs = String::from(log)
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
