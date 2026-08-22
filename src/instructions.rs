use std::{
    io::{Stdout, Write},
    time::Duration,
};

use crossterm::{cursor::MoveTo, event, execute};

use crate::helpers::{self, CUSTOM_FOREGROUND_COLOR, queue_centered_message};

pub fn run(stdout: &mut Stdout) -> Result<(), String> {
    helpers::render_border_over_duration(stdout, Duration::from_secs(1), true)?;
    let first_message_ended_row = queue_centered_message(
        "You'll be given a paragraph (that is not AI generated, mind you) and a minute to type out as much of it as possible.",
        stdout,
        2,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )?;
    let second_message_ended_row = queue_centered_message(
        "Exit commands: ESC or CTRL + C.",
        stdout,
        first_message_ended_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )?;
    let third_message_ended_row = queue_centered_message(
        "stats visible at 'heyo stats'",
        stdout,
        second_message_ended_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )?;
    queue_centered_message(
        "Press any key to start.",
        stdout,
        third_message_ended_row + 2,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )?;
    stdout.flush().map_err(|e| e.to_string())?;
    event::read().map_err(|e| e.to_string())?;
    execute!(stdout, MoveTo(0, 0)).map_err(|e| e.to_string())?;
    return Ok(());
}
