use std::io::{self, Write};
use std::println;

use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::{All, FromCursorDown};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Result<Self, String> {
        enable_raw_mode().map_err(|e| e.to_string())?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub fn host_race() -> Result<(), String> {
    let _raw_mode = RawModeGuard::new().map_err(|e| e.to_string())?;

    let mut stdout = io::stdout();
    let mut user_input = String::new();
    let final_sentence = "The brown fox jumps over bla";

    let mut cursor_index = 0;

    execute!(
        stdout,
        Clear(All),
        MoveTo(0, 2),
        Print(format!(
            "Your sentence: '{}'\n\r",
            final_sentence.to_string()
        )),
        Print("Start whenever you are ready!\n\r"),
    )
    .map_err(|e| e.to_string())?;

    render(&mut stdout, final_sentence, &user_input, cursor_index)?;
    // time starts

    loop {
        let event = event::read().map_err(|e| e.to_string())?;

        // redraw
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(ch) => {
                    user_input.insert(cursor_index, ch);
                    cursor_index += 1;
                }
                KeyCode::Backspace => {
                    if cursor_index > 0 {
                        cursor_index -= 1;
                        user_input.remove(cursor_index);
                    }
                }
                KeyCode::Left => {
                    if cursor_index > 0 {
                        cursor_index -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_index < user_input.len() {
                        cursor_index += 1;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            },
            _ => {}
        }
        render(&mut stdout, final_sentence, &user_input, cursor_index)?;
    }

    // end time

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    _final_sentence: &str,
    user_input: &str,
    cursor_index: usize,
) -> Result<(), String> {
    execute!(
        stdout,
        MoveTo(0, 5),
        Clear(FromCursorDown),
        MoveTo(0, 7),
        Print(user_input),
        MoveTo(cursor_index as u16, 7),
    )
    .map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    Ok(())
}
