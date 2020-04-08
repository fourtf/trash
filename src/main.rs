#![allow(unused_imports)]
use crossterm::{
    cursor::{MoveDown, MoveTo, MoveUp},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use phf::phf_map;
use std::cell::Cell;
use std::{
    env,
    ffi::OsString,
    io::{stdin, stdout, Write},
    process::Command,
};

#[cfg(windows)]
use winapi::{
    shared::minwindef::DWORD,
    um::{
        consoleapi::{GetConsoleMode, SetConsoleMode},
        processenv::GetStdHandle,
    },
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
        let mut line = CmdLine::new();
        line.print()?;

        while !event::poll(std::time::Duration::from_secs(1))? {}

        loop {
            match event::read()? {
                // exit
                Event::Key(KeyEvent {
                    code: KeyCode::Char('z'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    brint_debug("[Ctrl+z]")?;
                    return Ok(());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: _,
                }) => {
                    brint_debug("[Esc]")?;
                    return Ok(());
                }

                // cancel input
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    brint_debug("[Ctrl+C]\r\n")?;
                    break;
                }

                // enter -> execute command
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: _,
                }) => {
                    if !line.is_empty() {
                        let words: Vec<String> = line
                            .content()
                            .split_whitespace()
                            .map(|word| word.to_owned())
                            .collect();

                        if has_builtin(&words) {
                            exec_builtin(&words);
                        } else {
                            exec_native(&words);
                        }
                    }

                    break;
                }

                // misc
                e => {
                    line.handle_event(e)?;
                } // Event::Key(KeyEvent {
                  //     code: a,
                  //     modifiers: b,
                  // }) => {
                  //     // line.handle_event())

                  //     // brint_debug(format!("[{:?} {:?}]", a, b))?;
                  // }

                  // Event::Mouse(m) => {}
                  // Event::Resize(x, y) => {
                  //     brint_debug(format!("[resize: {} {}]", x, y))?;
                  // }
                  // _ => {}
            }
        }

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
    execute!(stdout(), Print(s))?;

    Ok(())
}

fn brint_debug<T: std::fmt::Display + Clone>(s: T) -> crossterm::Result<()> {
    brint_colored(s, Color::Green)
}

fn brint_resultln<T: std::fmt::Display + Clone>(s: T) -> crossterm::Result<()> {
    brint_colored(s, Color::Blue)
}

fn brint_errorln<T: std::fmt::Display + Clone>(s: T) -> crossterm::Result<()> {
    brint_colored(s, Color::Red)
}

fn brint_colored<T: std::fmt::Display + Clone>(s: T, color: Color) -> crossterm::Result<()> {
    execute!(stdout(), SetForegroundColor(color), Print(s), ResetColor)?;

    Ok(())
}

fn exec_native(words: &Vec<String>) {
    let words: Vec<OsString> = words.iter().map(|word| word.into()).collect();

    let os_cmd: std::ffi::OsString = words
        .get(0)
        .map(|x| x.to_owned().into())
        .unwrap_or("".into());

    brint("\r\n").ok();

    // run program
    exec_native_platform_dependant(&|| {
        disable_raw_mode().ok();
        let exit_status = Command::new(&os_cmd)
            .args(words.iter().skip(1))
            .spawn()
            .and_then(|mut c| c.wait());
        enable_raw_mode().ok();

        match exit_status {
            Ok(status) => {
                brint_resultln(format!("\r[{}]\r\n", status.code().unwrap_or(0))).ok();
            }
            Err(e) => {
                brint_errorln(format!("\r\n{}\r\n", e)).ok();
            }
        }
    })
}

#[cfg(not(windows))]
fn exec_native_platform_dependant(func: &dyn Fn()) {
    // func();
}

#[cfg(windows)]
fn exec_native_platform_dependant(func: &dyn Fn()) {
    unsafe {
        let handle = GetStdHandle((-11i64) as DWORD);
        let mode: DWORD = 0;
        GetConsoleMode(handle, std::mem::transmute(&mode));

        func();

        SetConsoleMode(handle, mode);
    }
}

#[derive(Default)]
struct CmdLine {
    content: String,
    line_count: Cell<usize>,
}

impl CmdLine {
    fn new() -> CmdLine {
        let line = CmdLine::default();
        line.line_count.set(1);
        return line;
    }

    fn print(&self) -> crossterm::Result<()> {
        let mut stdout = stdout();
        let (term_width, _term_height) = crossterm::terminal::size()?;
        let (_x, y) = crossterm::cursor::position()?;

        // assemble line to be drawn
        let mut words: Vec<(String, Color)> = Vec::new();

        words.push((pwd(), Color::Red));
        words.push(("> ".to_owned(), Color::Red));

        let chars: Vec<char> = self.content.chars().collect();
        let mut i = 0;

        let mut head = String::new();
        let mut cmd = String::new();
        let mut tail = String::new();

        while i < chars.len() {
            if chars[i].is_whitespace() {
                head.push(chars[i]);
            } else {
                while i < chars.len() {
                    if chars[i].is_whitespace() {
                        while i < chars.len() {
                            tail.push(chars[i]);
                            i += 1;
                        }
                    } else {
                        cmd.push(chars[i]);
                    }
                    i += 1;
                }
            }
            i += 1;
        }

        words.push((head, Color::Reset));
        words.push((
            cmd.clone(),
            if has_builtin_cmd(&cmd) || which::which(cmd).is_ok() {
                Color::Green
            } else {
                Color::Red
            },
        ));
        words.push((tail, Color::Reset));

        //words.push((self.content.clone(), Color::Reset));

        // calculate width
        let line_width: usize = words.iter().fold(0, |acc: usize, x| acc + x.0.len());
        let line_count: usize = ((line_width.max(1) - 1) / (term_width as usize)) + 1;

        // queue output
        for _ in 0..(self.line_count.get() - 1) {
            queue!(stdout, Clear(ClearType::CurrentLine), MoveUp(1))?;
        }

        queue!(
            stdout,
            MoveTo(0, y - self.line_count.get() as u16 + 1),
            Clear(ClearType::CurrentLine)
        )?;

        for word in words {
            queue!(stdout, SetForegroundColor(word.1), Print(word.0))?;
        }

        // write output
        stdout.flush()?;

        // save state
        self.line_count.set(line_count.max(1));

        Ok(())
    }

    fn handle_event(&mut self, e: crossterm::event::Event) -> crossterm::Result<()> {
        match e {
            // input
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: _,
            }) => {
                // brint(c)?;

                self.content.push(c);
                self.print()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: _,
            }) => {
                if !self.content.is_empty() {
                    // brint("\x08 \x08")?;

                    self.content.pop();
                    self.print()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn content(&self) -> &String {
        &self.content
    }
}

fn pwd() -> String {
    let mut string: String = env::current_dir()
        .map(|path| path.to_string_lossy().to_owned().to_string())
        .unwrap_or("".to_owned());

    replace_home_dir(&mut string);

    let path_components: Vec<&str> = string.split(std::path::MAIN_SEPARATOR).collect();
    let x_d: Vec<&str> = path_components
        .iter()
        .take(path_components.len() - 1)
        .map(|x| {
            if x.len() == 0 {
                ""
            } else if x.len() == 2 && x.chars().next().unwrap() == ':' {
                &x[0..1]
            } else {
                &x[0..1]
            }
        })
        .collect();

    let fmt: String = format!("{}", std::path::MAIN_SEPARATOR);

    x_d.join(&fmt) + &fmt + path_components.last().unwrap()
}

fn replace_home_dir(s: &mut String) {
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if s.starts_with(&home) {
        *s = String::from("~") + &s[home.len()..];
    }
}

//
// builtins
//

static BUILTINS: phf::Map<&'static str, fn(&Vec<String>)> = phf_map! {
    "exit" => exit,
    "cd" => cd,
    // "clear" => clear,
};

fn has_builtin(args: &Vec<String>) -> bool {
    match args.get(0) {
        Some(cmd) => has_builtin_cmd(&cmd),
        None => false,
    }
}

fn has_builtin_cmd(cmd: &String) -> bool {
    BUILTINS.contains_key(&cmd[..])
}

fn exec_builtin(args: &Vec<String>) -> bool {
    let cmd = args.get(0).map(|x| &x[..]).unwrap_or("");

    if BUILTINS.contains_key(cmd) {
        (BUILTINS[cmd])(args);

        true
    } else {
        false
    }
}

fn exit(_args: &Vec<String>) {
    std::process::exit(0);
}

fn cd(args: &Vec<String>) {
    brint("xD").ok();

    if let Some(to) = args.get(0) {
        std::env::set_current_dir(to).ok();
    } else if let Some(to) = dirs::home_dir() {
        std::env::set_current_dir(to).ok();
    }
}

// fn clear(_args: &Vec<String>) {
//     execute!(stdout(), crossterm::terminal::Clear(ClearType::All)).ok();
// }
