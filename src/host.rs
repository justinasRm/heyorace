use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::{println, todo};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::{All, FromCursorDown};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm::{execute, terminal};

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
    let typing_duration = Duration::from_secs(5);

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

    let race_started_at = Instant::now();

    render(
        &mut stdout,
        final_sentence,
        &user_input,
        cursor_index,
        race_started_at,
        typing_duration.as_secs_f64(),
    )?;
    // time starts
    loop {
        if race_started_at.elapsed() >= typing_duration {
            break;
        }
        render(
            &mut stdout,
            final_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
        )?;

        if !event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
            continue;
        }

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
        render(
            &mut stdout,
            final_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
        )?;
    }

    // end time

    todo!("Print diagnostics");

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    final_sentence: &str,
    user_input: &str,
    cursor_index: usize,
    race_started_at: Instant,
    total_seconds: f64,
) -> Result<(), String> {
    let input_start_col = 0;
    let input_start_row = 7;

    let (cursor_col, cursor_row) =
        cursor_position_for_index(cursor_index, input_start_col, input_start_row)?;

    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();
    execute!(
        stdout,
        MoveTo(0, 4),
        Print(format!(
            "Elapsed: {:.1}s / {:.1}s",
            elapsed_seconds, total_seconds
        )),
        MoveTo(0, 5),
        Clear(FromCursorDown),
        MoveTo(input_start_col, input_start_row),
        Print(user_input),
        MoveTo(cursor_col, cursor_row),
    )
    .map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    Ok(())
}

fn cursor_position_for_index(
    cursor_index: usize,
    input_start_col: u16,
    input_start_row: u16,
) -> Result<(u16, u16), String> {
    let (terminal_width, _) = terminal::size().map_err(|e| e.to_string())?;

    let absolute_col = input_start_col as usize + cursor_index;
    let row_offset = absolute_col / terminal_width as usize;
    let col = absolute_col % terminal_width as usize;

    Ok((col as u16, input_start_row + row_offset as u16))
}
