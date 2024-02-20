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
            process: String::from("fetching info..."),
        }
    }

    pub fn update_logs(&mut self, log: &str) {
        self.logs.push_str(log);
        self.logs.push_str("\n")
    }

    pub fn update_process(&mut self, info: &str) {
        self.process = String::from(info)
    }

    pub fn print_logs(&self) -> std::io::Result<()> {
        println!("{}", self.logs);
        Ok(())
    }
}
