use tui_input::Input;

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
    /// database connections
    pub process: String,
}

impl App {
    pub fn new(cmd: &str) -> App {
        App {
            input: Input::default(),
            input_mode: InputMode::Normal,
            cmd: String::from(cmd),
            logs: String::new(),
            scroll_pos: 0,
            process: String::from("fetching info..."),
        }
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
}
