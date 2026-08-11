use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::unreachable;

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
    let typing_duration = Duration::from_secs(6);

    let mut stdout = io::stdout();
    let mut user_input = String::new();
    let final_sentence = "The brown fox jumps over bla The brown fox jumps over bla";
    let mut cursor_row = 0;
    let mut cursor_index = 0;
    starting_message(&mut stdout, final_sentence)?;

    let race_started_at = Instant::now();

    cursor_row = render(
        &mut stdout,
        final_sentence,
        &user_input,
        cursor_index,
        race_started_at,
        typing_duration.as_secs_f64(),
    )?;

    loop {
        if race_started_at.elapsed() >= typing_duration {
            break;
        }
        cursor_row = render(
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
        cursor_row = render(
            &mut stdout,
            final_sentence,
            &user_input,
            cursor_index,
            race_started_at,
            typing_duration.as_secs_f64(),
        )?;
    }

    // end time

    let (wpm, accuracy) = statistics(final_sentence, &user_input.to_string(), typing_duration)?;

    print_statistics(wpm, accuracy, &mut stdout, cursor_row)?;

    return Ok(());
}

fn render(
    stdout: &mut std::io::Stdout,
    final_sentence: &str,
    user_input: &str,
    cursor_index: usize,
    race_started_at: Instant,
    total_seconds: f64,
) -> Result<u16, String> {
    let input_start_col = 0;
    let input_start_row = 7;

    let (cursor_col, cursor_row) =
        cursor_position_for_index(cursor_index, input_start_col, input_start_row)?;

    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();
    execute!(
        stdout,
        MoveTo(0, 5),
        Print(format!(
            "Elapsed: {:.1}s / {:.1}s",
            elapsed_seconds, total_seconds
        )),
        MoveTo(0, 6),
        Clear(FromCursorDown),
        MoveTo(input_start_col, input_start_row),
        Print(user_input),
        MoveTo(cursor_col, cursor_row),
    )
    .map_err(|e| e.to_string())?;

    stdout.flush().map_err(|e| e.to_string())?;

    Ok(cursor_row)
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
    current_cursor_row: u16,
) -> Result<(), String> {
    execute!(
        stdout,
        MoveTo(0, current_cursor_row + 1),
        Print("------"),
        MoveTo(0, current_cursor_row + 2),
        Print(format!("Words per minute: {:.1}", wpm)),
        MoveTo(0, current_cursor_row + 3),
        Print(format!("Accuracy: {:.1}", accuracy)),
        MoveTo(0, current_cursor_row + 4)
    )
    .map_err(|e| e.to_string())?;

    return Ok(());
}

fn starting_message(stdout: &mut std::io::Stdout, final_sentence: &str) -> Result<(), String> {
    let first_message: String = "The sentence is:".to_string();
    let first_message_column =
        centered_message_column(first_message.as_str()).map_err(|e| e.to_string())?;
    let second_message: String = format!("'{final_sentence}'");
    let second_message_column =
        centered_message_column(second_message.as_str()).map_err(|e| e.to_string())?;
    let third_message = "Start whenever you're ready!".to_string();
    let third_message_column =
        centered_message_column(third_message.as_str()).map_err(|e| e.to_string())?;
    execute!(
        stdout,
        Clear(All),
        MoveTo(first_message_column, 2),
        Print(first_message),
        MoveTo(second_message_column, 3),
        Print(second_message),
        MoveTo(third_message_column, 4),
        Print(third_message),
    )
    .map_err(|e| e.to_string())?;

    return Ok(());
}

// returns column index, so the provided 'message' is centered
fn centered_message_column(message: &str) -> Result<u16, String> {
    let (terminal_width, _) = terminal::size().map_err(|e| e.to_string())?;
    let col = terminal_width / 2 - message.len() as u16 / 2;

    // TODO: the text to input will be log and multiple lines. Make it support it.
    return Ok(col);
}
