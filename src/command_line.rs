use crossterm::{
    cursor::{MoveDown, MoveTo, MoveUp},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    write_ansi_code, ExecutableCommand,
};
use std::cell::Cell;
use std::{
    env,
    ffi::OsString,
    io::{stdin, stdout, Write},
    process::Command,
};

use super::builtins::has_builtin_cmd;
use super::parser::Token;

#[derive(Default)]
pub struct CmdLine {
    content: String,
    line_count: Cell<usize>,
}

impl CmdLine {
    pub fn new() -> CmdLine {
        let line = CmdLine::default();
        line.line_count.set(1);
        return line;
    }

    pub fn print(&self) -> crossterm::Result<()> {
        let mut stdout = stdout();
        let (term_width, _term_height) = crossterm::terminal::size()?;
        let (_x, y) = crossterm::cursor::position()?;

        // assemble line to be drawn
        let mut words: Vec<(String, Color)> = Vec::new();

        words.push((pwd(), Color::Red));
        words.push(("> ".to_owned(), Color::Red));

        for token in super::parser::tokenize_program_call(self.content.as_str()) {
            match token {
                Token::FunctionCall(s) => {
                    words.push((
                        s.to_owned(),
                        if has_builtin_cmd(&s.to_owned()) || which::which(s).is_ok() {
                            Color::Green
                        } else {
                            Color::Red
                        },
                    ));
                }
                Token::Argument(s) | Token::Space(s) => words.push((s.to_owned(), Color::Reset)),
                Token::Extra(s) => words.push((s.to_owned(), Color::Magenta)),
            }
        }

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

        queue!(stdout, ResetColor)?;

        // write output
        stdout.flush()?;

        // save state
        self.line_count.set(line_count.max(1));

        Ok(())
    }

    pub fn handle_event(&mut self, e: crossterm::event::Event) -> crossterm::Result<()> {
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

    pub fn is_empty(&self) -> bool {
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
