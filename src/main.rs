use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::enable_raw_mode,
    ExecutableCommand,
};
use std::{
    env,
    ffi::OsString,
    io::{stdin, stdout, Write},
    process::Command,
};

const APP_VERSION: &'static str = "0.1";
const APP_NAME: &'static str = "trash shell";

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);

    // Ctrl+C handler
    ctrlc::set_handler(move || {
        #[cfg(windows)]
        stdout().execute(Print("^C")).unwrap();
    })
    .unwrap_or_else(|_err| println!("Error initializing the Ctrl+C handler."));

    // Raw mode
    enable_raw_mode().unwrap_or_else(|_err| println!("Error enabling raw mode."));

    // Main loop
    run_loop().unwrap();
}

fn run_loop() -> crossterm::Result<()> {
    loop {
        // print!("[1]");
        print_status()?;

        while !event::poll(std::time::Duration::from_secs(1))? {}

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                brint("[Ctrl+C]")?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                brint("[Ctrl+a]")?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('A'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                brint("[Ctrl+A]")?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: _,
            }) => {
                std::process::exit(0);
            }
            Event::Key(KeyEvent {
                code: a,
                modifiers: b,
            }) => {
                brint(format!("[{:?} {:?}]", a, b))?;
            }
            // Event::Mouse(m) => {}
            // Event::Resize(x, y) => {}
            _ => {}
        }

        // print!("[2]");

        // let mut line = String::new();
        // print!("[1]");

        // stdin().read_line(&mut line)?;

        // let chars: Vec<char> = line.chars().collect();

        // print!("[len: {}]", line.len());
        // print!("[char_len: {}]", chars.len());
        // if chars.len() >= 1 {
        //     print!("[ord: {}]", chars[0].to_digit(10).unwrap_or(0));
        // }
        // print!("[2]");

        // println!("[echo: {}]", &line);

        // let words: Vec<String> = line
        //     .split_whitespace()
        //     .map(|word| word.to_owned())
        //     .collect();

        // if words.is_empty() {
        //     continue;
        // }

        // if words[0] == "exit" {
        //     break Ok(());
        // } else {
        //     exec_native(&words);
        // }
    }
}

fn brint<T: std::fmt::Display + Clone>(s: T) -> crossterm::Result<()> {
    execute!(
        stdout(),
        SetForegroundColor(Color::Green),
        Print(s),
        ResetColor
    )?;

    Ok(())
}

fn exec_native(words: &Vec<String>) {
    let words: Vec<OsString> = words.iter().map(|word| word.into()).collect();

    let os_cmd: std::ffi::OsString = words[0].to_owned().into();
    let exit_status = Command::new(&os_cmd)
        .args(words[1..].iter())
        .spawn()
        .and_then(|mut c| c.wait());

    match exit_status {
        Ok(status) => {
            print!("result: {:?}", status.code());
        }
        Err(e) => {
            print!("{}", e);
        }
    }
}

fn print_status() -> crossterm::Result<()> {
    execute!(
        stdout(),
        SetForegroundColor(Color::Red),
        Print("\r\n"),
        Print(pwd()),
        Print("> "),
        ResetColor
    )?;

    Ok(())
}

fn pwd() -> String {
    env::current_dir()
        .map(|path| path.to_string_lossy().to_owned().to_string())
        .unwrap_or("".to_owned())
}
