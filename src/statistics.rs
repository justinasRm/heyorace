use std::{io::Write, thread::sleep, time::Duration};

use crossterm::{cursor::MoveTo, execute, queue, style::Print, terminal};

use crate::helpers::{self, CUSTOM_FOREGROUND_COLOR, queue_centered_message};

pub fn statistics_display_from_args(stdout: &mut std::io::Stdout) -> Result<(), String> {
    let (_, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    helpers::render_border(stdout, true)?;

    let last_wpm = 55.5;
    let last_accuracy = 69.7;

    queue_centered_message(
        format!(
            "Your last result: WPM {:.1}, accuracy {:.1}%.",
            last_wpm, last_accuracy
        )
        .as_str(),
        stdout,
        2,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;

    stdout.flush().map_err(|e| e.to_string())?;
    sleep(Duration::from_secs(5));
    execute!(stdout, MoveTo(0, terminal_height - 1), Print("\r\n")).map_err(|e| e.to_string())?;

    return Ok(());
}

pub fn print_after_race_statistics(
    wpm: f64,
    accuracy: f64,
    stdout: &mut std::io::Stdout,
) -> Result<(), String> {
    let (_, terminal_height) = terminal::size().map_err(|e| e.to_string())?;

    helpers::queue_centered_message(
        "Finished!",
        stdout,
        terminal_height - 4,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )
    .map_err(|e| e.to_string())?;
    helpers::queue_centered_message(
        format!("WPM: {:.1}, accuracy: {:.1}%.", wpm, accuracy).as_str(),
        stdout,
        terminal_height - 3,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )
    .map_err(|e| e.to_string())?;
    helpers::queue_centered_message(
        "Stats updated. Check with 'heyo stats'.",
        stdout,
        terminal_height - 2,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )
    .map_err(|e| e.to_string())?;

    queue!(stdout, MoveTo(0, terminal_height - 1), Print("\r\n")).map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

pub fn save_statistics(wpm: f64, accuracy: f64) -> Result<(), String> {
    //
    return Ok(());
}
