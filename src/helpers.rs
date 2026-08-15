use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::FromCursorDown;
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

    let reveal_duration = Duration::from_secs(2);

    let countdown_started_at = Instant::now();
    let mut last_displayed_second: Option<u64> = None;

    for (char_index, correct_char) in correct_sentence.bytes().enumerate() {
        let target_elapsed = Duration::from_secs_f64(
            reveal_duration.as_secs_f64() * char_index as f64 / correct_sentence.len() as f64,
        );
        let actual_elapsed = countdown_started_at.elapsed();
        if target_elapsed > actual_elapsed {
            sleep(target_elapsed - actual_elapsed);
        }

        let elapsed_seconds = countdown_started_at.elapsed().as_secs();

        if last_displayed_second != Some(elapsed_seconds) {
            last_displayed_second = Some(elapsed_seconds);

            let countdown_message = match elapsed_seconds {
                0 => "3...".to_string(),
                1 => "3... 2...".to_string(),
                _ => "NOREACH".to_string(),
            };

            queue_centered_message(
                &countdown_message,
                stdout,
                starting_row + 1,
                Some(CUSTOM_FOREGROUND_COLOR),
                None,
                true,
            )?;
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
