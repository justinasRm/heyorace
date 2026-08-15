use std::io::{Stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::style::{
    Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::FromCursorDown;
use crossterm::{execute, queue, terminal};
use std::sync::atomic::{AtomicU16, Ordering};

pub static USABLE_TERMINAL_WIDTH: AtomicU16 = AtomicU16::new(0);
pub static USABLE_TERMINAL_HEIGHT: AtomicU16 = AtomicU16::new(0);
pub const LEFT_OFFSET: usize = 2;
pub const CUSTOM_BACKGROUND_COLOR: crossterm::style::Color = Color::Rgb {
    r: 16,
    g: 24,
    b: 32,
};
pub const CUSTOM_FOREGROUND_COLOR: crossterm::style::Color = Color::Rgb {
    r: 254,
    g: 231,
    b: 21,
};
pub const CUSTOM_FOREGROUND_COLOR_DARK: crossterm::style::Color = Color::Rgb {
    r: 122,
    g: 110,
    b: 1,
};
pub const CUSTOM_ERROR_COLOR: crossterm::style::Color = Color::Rgb {
    r: 220,
    g: 50,
    b: 47,
};

pub fn print_countdown(
    stdout: &mut std::io::Stdout,
    total_seconds: f64,
    starting_row: u16,
    correct_sentence: &str,
    usable_terminal_width: u16,
    input_start_col: u16,
    input_start_row: u16,
) -> Result<(), String> {
    let elapsed_str = format!("Elapsed: {:.1}s / {:.1}s", 0, total_seconds);
    queue!(stdout, Clear(FromCursorDown)).map_err(|e| e.to_string())?;

    queue_centered_message(
        elapsed_str.as_str(),
        stdout,
        starting_row + 3,
        None,
        None,
        false,
    )?;

    let countdown_started_at = Instant::now();
    queue_centered_message(
        "3...",
        stdout,
        starting_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;
    stdout.flush().map_err(|e| e.to_string())?;

    for (char_index, correct_char) in correct_sentence.bytes().enumerate() {
        let target_elapsed =
            Duration::from_secs_f64(char_index as f64 / correct_sentence.len() as f64);
        let actual_elapsed = countdown_started_at.elapsed();
        if target_elapsed > actual_elapsed {
            sleep(target_elapsed - actual_elapsed);
        }

        let row_offset = char_index / usable_terminal_width as usize;
        let col_offset = char_index % usable_terminal_width as usize;

        queue!(
            stdout,
            MoveTo(
                input_start_col + col_offset as u16,
                input_start_row + row_offset as u16,
            ),
            SetForegroundColor(CUSTOM_FOREGROUND_COLOR_DARK),
            Print(correct_char as char),
        )
        .map_err(|e| e.to_string())?;

        stdout.flush().map_err(|e| e.to_string())?;
    }

    queue_centered_message(
        "3... 2...",
        stdout,
        starting_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;
    queue!(stdout, MoveTo(input_start_col, input_start_row)).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;
    sleep(Duration::from_secs(1));

    queue_centered_message(
        "3... 2... 1...",
        stdout,
        starting_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;
    queue!(stdout, MoveTo(input_start_col, input_start_row)).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;
    sleep(Duration::from_secs(1));

    queue_centered_message(
        "3... 2... 1... GO!!!",
        stdout,
        starting_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;
    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

// respects cli border, centers
pub fn queue_centered_message(
    message: &str,
    stdout: &mut std::io::Stdout,
    starting_row: u16,
    text_color: Option<Color>,
    background_color: Option<Color>,
    bold: bool,
) -> Result<u16, String> {
    let usable_terminal_width = USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed);
    let mut total_lines = 0;
    let chunks_count = message
        .as_bytes()
        .chunks(usable_terminal_width as usize)
        .len();
    let message_chunks = message
        .as_bytes()
        .chunks(usable_terminal_width as usize)
        .enumerate();

    for (line_index, chunk) in message_chunks {
        let line = std::str::from_utf8(chunk).map_err(|e: std::str::Utf8Error| e.to_string())?;
        total_lines += 1;

        let mut input_col = LEFT_OFFSET as u16;
        if line_index < chunks_count {
            input_col = usable_terminal_width / 2 - (chunk.len() as u16 / 2) + LEFT_OFFSET as u16;
        }
        queue!(
            stdout,
            MoveTo(input_col, starting_row + line_index as u16),
            SetAttribute(if bold {
                crossterm::style::Attribute::Bold
            } else {
                crossterm::style::Attribute::Reset
            }),
            SetBackgroundColor(background_color.unwrap_or(Color::Reset)),
            SetForegroundColor(text_color.unwrap_or(Color::Reset)),
            Print(line),
            SetAttribute(crossterm::style::Attribute::Reset),
        )
        .map_err(|e| e.to_string())?;
    }

    return Ok(total_lines + starting_row - 1);
}

pub fn render_border(stdout: &mut Stdout) -> Result<(), String> {
    let (terminal_width, terminal_height) = terminal::size().map_err(|e| e.to_string())?;

    let horizontal_border = "-".repeat(terminal_width as usize);

    queue!(
        stdout,
        SetForegroundColor(Color::Black),
        MoveTo(0, 1),
        Print(&horizontal_border),
        MoveTo(0, terminal_height),
        Print(&horizontal_border),
    )
    .map_err(|e| e.to_string())?;

    for row in 1..terminal_height {
        queue!(stdout, MoveTo(0, row), Print('|')).map_err(|e| e.to_string())?;
        queue!(stdout, MoveTo(terminal_width, row), Print('|')).map_err(|e| e.to_string())?;
    }
    queue!(stdout, ResetColor).map_err(|e| e.to_string())?;

    return Ok(());
}

pub fn render_border_over_duration(stdout: &mut Stdout, duration: Duration) -> Result<(), String> {
    let (terminal_width, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    let total_chars_to_print: u16 = terminal_width * 2 + terminal_height * 2;
    let sleep_timer = duration.as_secs_f64() / total_chars_to_print as f64;

    execute!(stdout, SetForegroundColor(Color::Black)).map_err(|e| e.to_string())?;

    for column in 0..terminal_width {
        execute!(stdout, MoveTo(column, 1), Print('-')).map_err(|e| e.to_string())?;
        sleep(Duration::from_secs_f64(sleep_timer));
    }
    for row in 1..terminal_height {
        execute!(stdout, MoveTo(terminal_width, row), Print('|')).map_err(|e| e.to_string())?;
        sleep(Duration::from_secs_f64(sleep_timer));
    }
    for column in 1..terminal_width {
        execute!(
            stdout,
            MoveTo(terminal_width - column - 1, terminal_height),
            Print('-')
        )
        .map_err(|e| e.to_string())?;
        sleep(Duration::from_secs_f64(sleep_timer));
    }
    for row in 1..terminal_height {
        execute!(stdout, MoveTo(0, terminal_height - row), Print('|'))
            .map_err(|e| e.to_string())?;
        sleep(Duration::from_secs_f64(sleep_timer));
    }

    execute!(stdout, ResetColor).map_err(|e| e.to_string())?;

    return Ok(());
}
