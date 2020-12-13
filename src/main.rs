#![allow(unused_imports)]
use crossterm::{
    cursor::{MoveDown, MoveTo, MoveUp},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    write_ansi_code, ExecutableCommand,
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

mod command_line;
use command_line::CmdLine;
mod builtins;
mod parser;
use builtins::{exec_builtin, has_builtin, has_builtin_cmd};

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
                        match parser::parse_program_call(line.content().as_str()) {
                            Ok((_, call)) => {
                                let mut words = vec![call.program];
                                words.append(&mut call.args.clone());

                                if has_builtin(&words) {
                                    exec_builtin(&words);
                                } else {
                                    exec_native(&words);
                                }
                            }
                            Err(_) => (),
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
    func();
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
