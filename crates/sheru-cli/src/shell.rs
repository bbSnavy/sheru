use std::{env, io};
use std::env::VarError;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{Command, exit, Stdio};
use std::sync::Mutex;
use is_executable::is_executable;

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
                    return None;
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

        if name == "exit" {
            println!("exiting");
            exit(0);
        }

        let mut paths = Vec::new();
        paths.push(name.clone());
        if let Ok(path_list) = env::var("PATH") {
            for path_elem in path_list.split(':') {
                let mut path_entry = String::new();
                path_entry.push_str(path_elem);
                path_entry.push('/');
                path_entry.push_str(name.clone().as_str());
                paths.push(path_entry);
            }
        }

        paths = paths.iter()
            .cloned()
            .filter(|p| {
                is_executable(Path::new(p))
            })
            .collect();

        match paths.first() {
            None => {
                eprintln!("command not found");
                self.flush();
                return;
            }
            Some(v) => {
                match Command::new(v)
                    .args(&args.clone()[1..])
                    .stdin(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .output() {
                    Ok(_) => {

                    },
                    Err(_) => {
                        eprintln!("failed at executing");
                    }
                }
            }
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
                    eprintln!("\nfailed at reading input");
                    break;
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
