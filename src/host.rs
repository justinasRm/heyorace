use std::io::{self, Write};
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
use crossterm::{queue, terminal};
use std::sync::atomic::{AtomicU16, Ordering};

static USABLE_TERMINAL_WIDTH: AtomicU16 = AtomicU16::new(0);
static USABLE_TERMINAL_HEIGHT: AtomicU16 = AtomicU16::new(0);
static LEFT_OFFSET: AtomicU16 = AtomicU16::new(2);
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
    let typing_duration = Duration::from_secs(5);
    set_initial_terminal_dimensions()?;

    let mut stdout = io::stdout();
    let mut user_input = String::new();
    let final_sentence = "The brown fox jumps over bla one to three four five six seven eight nine ten eleven twelve";
    let mut cursor_index = 0;
    let main_render_start_row = starting_message(&mut stdout, final_sentence)? + 1;

    let race_started_at = Instant::now();

    render(
        &mut stdout,
        final_sentence,
        &user_input,
        cursor_index,
        race_started_at,
        typing_duration.as_secs_f64(),
        main_render_start_row,
    )?;

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
            main_render_start_row,
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
            Event::Resize(w, h) => {
                USABLE_TERMINAL_WIDTH.store(w - 4, Ordering::Relaxed);
                USABLE_TERMINAL_HEIGHT.store(h - 2, Ordering::Relaxed);
            }
            _ => {}
        }

        render(
            &mut stdout,
            final_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
            main_render_start_row,
        )?;
    }

    let (wpm, accuracy) = statistics(final_sentence, &user_input.to_string(), typing_duration)?;

    print_statistics(wpm, accuracy, &mut stdout)?;

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    final_sentence: &str,
    user_input: &str,
    cursor_index: usize,
    race_started_at: Instant,
    total_seconds: f64,
    main_render_start_row: u16,
) -> Result<u16, String> {
    let input_start_col = 2;
    let mut first_free_row = main_render_start_row;
    let usable_terminal_width = USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed);
    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();

    queue!(
        stdout,
        MoveTo(2, first_free_row + 1),
        Print(format!(
            "Elapsed: {:.1}s / {:.1}s",
            elapsed_seconds, total_seconds
        )),
        MoveTo(2, first_free_row + 2),
        Clear(FromCursorDown)
    )
    .map_err(|e| e.to_string())?;
    first_free_row += 3;

    let (cursor_col, cursor_row) = cursor_position(
        cursor_index,
        input_start_col,
        first_free_row,
        usable_terminal_width,
    )?;

    render_border(stdout)?;
    render_user_typing(
        stdout,
        user_input,
        input_start_col,
        first_free_row,
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
    input_start_col: u16,
    input_start_row: u16,
    usable_terminal_width: u16,
) -> Result<(), String> {
    //
    for (line_index, chunk) in user_input
        .as_bytes()
        .chunks(usable_terminal_width as usize)
        .enumerate()
    {
        let line = std::str::from_utf8(chunk).map_err(|e| e.to_string())?;
        queue!(
            stdout,
            MoveTo(input_start_col, input_start_row + line_index as u16),
            Print(line),
        )
        .map_err(|e| e.to_string())?;
    }

    queue!(stdout, MoveTo(2, 5),).map_err(|e| e.to_string())?;

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
    final_sentence: &str,
    user_input: &str,
    total_duration: Duration,
) -> Result<(f64, f64), String> {
    let mut correct_word_count = 0;
    let mut accuracy = 100.0;

    let user_input_words: Vec<&str> = user_input.split(" ").collect();
    let total_word_count = user_input_words.len();
    let final_sentence_words: Vec<&str> = final_sentence.split(" ").collect();

    for (i, user_inputed_word) in user_input_words.iter().enumerate() {
        //
        let Some(correct_word) = final_sentence_words.get(i) else {
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

fn starting_message(stdout: &mut std::io::Stdout, final_sentence: &str) -> Result<u16, String> {
    queue!(stdout, Clear(All)).map_err(|e| e.to_string())?;

    let first_message: String = "The sentence is:".to_string();
    let first_message_ended_at_row = queue_centered_message(
        first_message.as_str(),
        stdout,
        2,
        // #02343F
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        false,
    )
    // 249, 235, 222
    .map_err(|e| e.to_string())?;
    let second_message: String = format!("'{final_sentence}'");
    let second_message_ended_at_row = queue_centered_message(
        second_message.as_str(),
        stdout,
        first_message_ended_at_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        true,
    )
    .map_err(|e| e.to_string())?;
    let third_message = "Start whenever you're ready!".to_string();
    let third_message_ended_at_row = queue_centered_message(
        third_message.as_str(),
        stdout,
        second_message_ended_at_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        Some(CUSTOM_BACKGROUND_COLOR),
        false,
    )
    .map_err(|e| e.to_string())?;

    // queue!(stdout, Clear(All)).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(third_message_ended_at_row);
}

// Need to rework - maybe a function that takes in strings, takes the starting row, and prints
// them all out centered and returns the final (column, row)?

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

        // if there is a next line, input_col = LEFT_OFFSET.load(Ordering::Relaxed);
        let mut input_col = LEFT_OFFSET.load(Ordering::Relaxed);
        if line_index < chunks_count {
            input_col = usable_terminal_width / 2 - (chunk.len() as u16 / 2)
                + LEFT_OFFSET.load(Ordering::Relaxed);
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
        )
        .map_err(|e| e.to_string())?;
    }

    return Ok(total_lines + starting_row - 1);
}
