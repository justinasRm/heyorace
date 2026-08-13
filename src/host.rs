use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::unreachable;

use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType::{All, FromCursorDown};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm::{execute, queue, terminal};

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
    let (terminal_width, _) = terminal::size().map_err(|e| e.to_string())?;
    let usable_terminal_width = terminal_width - 4;

    let mut stdout = io::stdout();
    let mut user_input = String::new();
    let final_sentence =
        "The brown fox jumps over bla The brown fox jumps fox jumps over bla over bla bla blaaaa";
    let mut cursor_index = 0;
    starting_message(&mut stdout, final_sentence, usable_terminal_width)?;

    let race_started_at = Instant::now();

    render(
        &mut stdout,
        final_sentence,
        &user_input,
        cursor_index,
        race_started_at,
        typing_duration.as_secs_f64(),
        usable_terminal_width,
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
            usable_terminal_width,
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
            usable_terminal_width,
        )?;
    }

    let (wpm, accuracy) = statistics(final_sentence, &user_input.to_string(), typing_duration)?;

    print_statistics(wpm, accuracy, &mut stdout, usable_terminal_width)?;

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    final_sentence: &str,
    user_input: &str,
    cursor_index: usize,
    race_started_at: Instant,
    total_seconds: f64,
    usable_terminal_width: u16,
) -> Result<u16, String> {
    let input_start_col = 2;
    let input_start_row = 7;

    let (cursor_col, cursor_row) = cursor_position(
        cursor_index,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();
    queue!(
        stdout,
        MoveTo(2, 5),
        Print(format!(
            "Elapsed: {:.1}s / {:.1}s",
            elapsed_seconds, total_seconds
        )),
        MoveTo(2, 6),
        Clear(FromCursorDown)
    )
    .map_err(|e| e.to_string())?;

    render_border(stdout)?;
    render_wrapped_text(
        stdout,
        user_input,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    queue!(stdout, MoveTo(cursor_col, cursor_row)).map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    Ok(cursor_row)
}

fn render_wrapped_text(
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

fn print_statistics(
    wpm: f64,
    accuracy: f64,
    stdout: &mut std::io::Stdout,
    usable_terminal_width: u16,
) -> Result<(), String> {
    let (_, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    // le
    let first_message: String = "Finished!".to_string();
    let first_message_column =
        centered_message_column(first_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;
    let second_message: String = format!("Words per minute: {:.1}", wpm).to_string();
    let second_message_column =
        centered_message_column(second_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;
    let third_message: String = format!("Accuracy: {:.1}%", accuracy).to_string();
    let third_message_column =
        centered_message_column(third_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;

    queue!(
        stdout,
        MoveTo(first_message_column, terminal_height - 4),
        Print(first_message),
        MoveTo(second_message_column, terminal_height - 3),
        Print(second_message),
        MoveTo(third_message_column, terminal_height - 2),
        Print(third_message),
        MoveTo(0, terminal_height - 1),
        Print("\r\n"),
    )
    .map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

fn starting_message(
    stdout: &mut std::io::Stdout,
    final_sentence: &str,
    usable_terminal_width: u16,
) -> Result<(), String> {
    let first_message: String = "The sentence is:".to_string();
    let first_message_column =
        centered_message_column(first_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;
    let second_message: String = format!("'{final_sentence}'");
    let second_message_column =
        centered_message_column(second_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;
    let third_message = "Start whenever you're ready!".to_string();
    let third_message_column =
        centered_message_column(third_message.as_str(), usable_terminal_width)
            .map_err(|e| e.to_string())?;
    execute!(
        stdout,
        Clear(All),
        MoveTo(first_message_column, 2),
        Print(first_message),
        SetForegroundColor(Color::Magenta),
        MoveTo(second_message_column, 3),
        Print(second_message),
        ResetColor,
        MoveTo(third_message_column, 4),
        Print(third_message),
    )
    .map_err(|e| e.to_string())?;

    return Ok(());
}

// returns column index, so the provided 'message' is centered
fn centered_message_column(message: &str, usable_terminal_width: u16) -> Result<u16, String> {
    let col = usable_terminal_width / 2 - message.len() as u16 / 2;
    // 88 / 2 - 88 / 2
    // TODO: the text to input will be long and multiple lines. Make it support it.
    return Ok(col);
}
