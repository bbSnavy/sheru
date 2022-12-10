use std::io;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;

pub struct Shell {
    isatty: Mutex<bool>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            isatty: Mutex::new(
                atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)),
        }
    }

    pub fn isatty(&self) -> bool {
        match self.isatty.lock() {
            Ok(v) => {
                *v
            }
            Err(_) => {
                false
            }
        }
    }

    pub fn flush(&self) {
        io::stdout().flush().unwrap();
        io::stderr().flush().unwrap();
    }

    pub fn prompt<S: Into<String>>(&self, s: S) {
        if self.isatty() {
            print!("{}", s.into());
            self.flush();
        }
    }

    pub fn prompt_input(&self) {
        self.prompt("sheru $ ");
    }

    pub fn prompt_line(&self) {
        self.prompt("sheru > ");
    }

    pub fn read_input_atty(&self) -> Option<Vec<String>> {
        let mut input = Vec::new();
        let mut stdin = io::stdin().lock();

        loop {
            let mut buf = String::new();

            match stdin.read_line(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    if let Some(v) = buf.strip_suffix('\n') {
                        buf = v.into();
                    }

                    if let Some(v) = buf.strip_suffix('\\') {
                        input.push(v.into());
                        continue;
                    }

                    input.push(buf);
                    break;
                }
                Err(err) => {
                    return None;
                }
            }
        }

        Some(input)
    }

    pub fn read_input_raw(&self) -> Option<Vec<String>> {
        todo!();

        None
    }

    pub fn read_input(&self) -> Option<Vec<String>> {
        match self.isatty() {
            true => self.read_input_atty(),
            false => self.read_input_raw(),
        }
    }

    pub fn process_input_command(&self, command: String) {
        let mut args = Vec::new();
        let mut name: String;

        for arg in command.split_whitespace() {
            args.push(arg.to_string());
        }

        match args.first() {
            Some(v) => {
                name = v.into();
            }
            None => {
                return;
            }
        }

        match Command::new(name)
            .args(&args.clone()[1..])
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output() {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    pub fn process_input_commands(&self, commands: Vec<String>) {
        for command in commands {
            self.process_input_command(command);
        }
    }

    pub fn process_input_line(&self, index: usize, line: String) {
        let mut commands = Vec::new();

        for command in line.split(';') {
            if command.trim().is_empty() {
                continue;
            }

            commands.push(command.to_string());
        }

        self.process_input_commands(commands);
    }

    pub fn process_input(&self, lines: Vec<String>) {
        for (i, v) in lines.iter().enumerate() {
            self.process_input_line(i, v.clone());
        }
    }

    pub fn start(&self) {
        loop {
            self.prompt_input();

            match self.read_input() {
                None => {
                    eprintln!("failed at reading input");
                }
                Some(lines) => {
                    self.process_input(lines);
                }
            }
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Shell::new()
    }
}