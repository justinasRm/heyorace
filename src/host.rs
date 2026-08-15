use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::unreachable;

use crossterm::cursor::MoveTo;
use crossterm::style::{
    Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::{All, FromCursorDown};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm::{execute, queue, terminal};
use std::sync::atomic::{AtomicU16, Ordering};

use crate::get_sentence;

static USABLE_TERMINAL_WIDTH: AtomicU16 = AtomicU16::new(0);
static USABLE_TERMINAL_HEIGHT: AtomicU16 = AtomicU16::new(0);
const LEFT_OFFSET: usize = 2;
const CUSTOM_BACKGROUND_COLOR: crossterm::style::Color = Color::Rgb {
    r: 16,
    g: 24,
    b: 32,
};
const CUSTOM_FOREGROUND_COLOR: crossterm::style::Color = Color::Rgb {
    r: 254,
    g: 231,
    b: 21,
};
const CUSTOM_FOREGROUND_COLOR_DARK: crossterm::style::Color = Color::Rgb {
    r: 122,
    g: 110,
    b: 1,
};
const CUSTOM_ERROR_COLOR: crossterm::style::Color = Color::Rgb {
    r: 220,
    g: 50,
    b: 47,
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

fn set_initial_terminal_dimensions() -> Result<(), String> {
    let (terminal_width, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    let usable_terminal_width = terminal_width - 4;
    let usable_terminal_height = terminal_height - 2;
    USABLE_TERMINAL_WIDTH.store(usable_terminal_width, Ordering::Relaxed);
    USABLE_TERMINAL_HEIGHT.store(usable_terminal_height, Ordering::Relaxed);

    return Ok(());
}

pub fn host_race() -> Result<(), String> {
    let _raw_mode = RawModeGuard::new().map_err(|e| e.to_string())?;
    let typing_duration = Duration::from_secs(60);
    set_initial_terminal_dimensions()?;

    let mut stdout = io::stdout();
    execute!(&mut stdout, Clear(All)).map_err(|e| e.to_string())?;
    render_border(&mut stdout)?;
    let mut user_input = String::new();
    let correct_sentence = &get_sentence::sentence();
    let mut cursor_index = 0;
    print_countdown(
        &mut stdout,
        typing_duration.as_secs_f64(),
        1,
        correct_sentence,
        USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed),
        LEFT_OFFSET as u16,
        6,
    )?;

    let race_started_at = Instant::now();

    loop {
        if race_started_at.elapsed() >= typing_duration {
            break;
        }
        render(
            &mut stdout,
            correct_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
            3,
        )?;

        if !event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
            continue;
        }

        let event = event::read().map_err(|e| e.to_string())?;

        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(ch) => {
                    if ch.is_ascii() {
                        user_input.insert(cursor_index, ch);
                        cursor_index += 1;
                    }
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
            Event::Resize(w, h) => {
                USABLE_TERMINAL_WIDTH.store(w - 4, Ordering::Relaxed);
                USABLE_TERMINAL_HEIGHT.store(h - 2, Ordering::Relaxed);
            }
            _ => {}
        }

        render(
            &mut stdout,
            correct_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
            3,
        )?;
    }

    let (wpm, accuracy) = statistics(correct_sentence, &user_input.to_string(), typing_duration)?;

    print_statistics(wpm, accuracy, &mut stdout)?;

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    correct_sentence: &str,
    user_input: &str,
    cursor_index: usize,
    race_started_at: Instant,
    total_seconds: f64,
    first_free_row: u16,
) -> Result<u16, String> {
    let input_start_col = 2;
    let input_start_row = first_free_row + 3;
    let usable_terminal_width = USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed);
    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();
    let elapsed_str = format!("Elapsed: {:.1}s / {:.1}s", elapsed_seconds, total_seconds);
    queue!(stdout, Clear(FromCursorDown)).map_err(|e| e.to_string())?;

    // elapsed_str is centered. Maybe on left I could show WPM, on right I could show accuracy in real time?
    queue_centered_message(
        elapsed_str.as_str(),
        stdout,
        // leaving space, so + 1.
        first_free_row + 1,
        None,
        None,
        false,
    )?;

    let (cursor_col, cursor_row) = cursor_position(
        cursor_index,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    render_border(stdout)?;

    render_user_typing(
        stdout,
        user_input,
        correct_sentence,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    queue!(stdout, MoveTo(cursor_col, cursor_row)).map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    Ok(cursor_row)
}

// respects cli border, doesnt center.
fn render_user_typing(
    stdout: &mut std::io::Stdout,
    user_input: &str,
    correct_sentence: &str,
    input_start_col: u16,
    input_start_row: u16,
    usable_terminal_width: u16,
) -> Result<(), String> {
    // first rendering all correct_sentence lines for the (under text?) background, so its easier to type on top of it
    for (line_index, chunk) in correct_sentence
        .as_bytes()
        .chunks(usable_terminal_width as usize)
        .enumerate()
    {
        let line = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
        queue!(
            stdout,
            MoveTo(input_start_col, input_start_row + line_index as u16),
            SetForegroundColor(CUSTOM_FOREGROUND_COLOR_DARK),
            Print(line),
            SetForegroundColor(Color::Reset)
        )
        .map_err(|e| e.to_string())?;
    }
    let correct_chars = correct_sentence.as_bytes();
    let correct_sentence_lines = correct_chars
        .chunks(usable_terminal_width as usize)
        .collect::<Vec<&[u8]>>();

    // then the real user input on top of it.
    for (line_index, chunk) in user_input
        .as_bytes()
        .chunks(usable_terminal_width as usize)
        .enumerate()
    {
        for (inputting_index, inputting_char) in chunk.iter().enumerate() {
            let mut is_inputed_char_correct: bool = false;
            if line_index < correct_sentence_lines.len() {
                let correct_line = correct_sentence_lines[line_index];
                if inputting_index < correct_line.len() {
                    let correct_char = correct_line[inputting_index];
                    if correct_char == *inputting_char {
                        is_inputed_char_correct = true;
                    }
                }
            }

            queue!(
                stdout,
                MoveTo(
                    input_start_col + inputting_index as u16,
                    input_start_row + line_index as u16
                ),
                SetForegroundColor(if is_inputed_char_correct {
                    CUSTOM_FOREGROUND_COLOR
                } else {
                    CUSTOM_ERROR_COLOR
                }),
                Print(*inputting_char as char)
            )
            .map_err(|e| e.to_string())?;
        }
    }

    queue!(stdout, MoveTo(2, 5), SetForegroundColor(Color::Reset)).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

fn render_border(stdout: &mut std::io::Stdout) -> Result<(), String> {
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

fn cursor_position(
    cursor_index: usize,
    input_start_col: u16,
    input_start_row: u16,
    usable_terminal_width: u16,
) -> Result<(u16, u16), String> {
    let usable_width = usable_terminal_width as usize;

    let row_offset = cursor_index / usable_width;
    let col_offset = cursor_index % usable_width;

    Ok((
        input_start_col + col_offset as u16,
        input_start_row + row_offset as u16,
    ))
}

fn statistics(
    correct_sentence: &str,
    user_input: &str,
    total_duration: Duration,
) -> Result<(f64, f64), String> {
    let mut correct_word_count = 0;
    let mut accuracy = 100.0;

    let user_input_words: Vec<&str> = user_input.split(" ").collect();
    let total_word_count = user_input_words.len();
    let correct_sentence_words: Vec<&str> = correct_sentence.split(" ").collect();

    for (i, user_inputed_word) in user_input_words.iter().enumerate() {
        //
        let Some(correct_word) = correct_sentence_words.get(i) else {
            unreachable!(
                "Final sentence should be long enough so user doesn't ever reach the end of it"
            );
        };

        if correct_word == user_inputed_word {
            correct_word_count += 1;
        }

        accuracy = (correct_word_count as f64 / total_word_count as f64) * 100.0;
    }

    let wpm: f64 = total_word_count as f64 / (total_duration.as_secs_f64() / 60.0);

    return Ok((wpm, accuracy));
}

fn print_statistics(wpm: f64, accuracy: f64, stdout: &mut std::io::Stdout) -> Result<(), String> {
    let (_, terminal_height) = terminal::size().map_err(|e| e.to_string())?;

    let first_message: String = "Finished!".to_string();
    queue_centered_message(
        first_message.as_str(),
        stdout,
        terminal_height - 4,
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        false,
    )
    .map_err(|e| e.to_string())?;
    let second_message: String = format!("Words per minute: {:.1}", wpm).to_string();
    queue_centered_message(
        second_message.as_str(),
        stdout,
        terminal_height - 3,
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        true,
    )
    .map_err(|e| e.to_string())?;
    let third_message: String = format!("Accuracy: {:.1}%", accuracy).to_string();
    queue_centered_message(
        third_message.as_str(),
        stdout,
        terminal_height - 2,
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        true,
    )
    .map_err(|e| e.to_string())?;

    queue!(stdout, MoveTo(0, terminal_height - 1), Print("\r\n"),).map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

fn print_countdown(
    stdout: &mut std::io::Stdout,
    total_seconds: f64,
    starting_row: u16,
    correct_sentence: &str,
    usable_terminal_width: u16,
    input_start_col: u16,
    input_start_row: u16,
) -> Result<(), String> {
    // FROM
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
fn queue_centered_message(
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
