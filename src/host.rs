use std::io::{Stdout, Write};
use std::time::{Duration, Instant};
use std::unreachable;

use crossterm::cursor::{self, MoveTo};
use crossterm::event::{self, Event, KeyCode};
use crossterm::queue;
use crossterm::style::{Color, Print, SetForegroundColor};
use std::sync::atomic::Ordering;

use crate::helpers::{CUSTOM_FOREGROUND_COLOR, move_to_end_before_exit};
use crate::{get_sentence, helpers, statistics};

pub fn solo_typing_speed_test(stdout: &mut Stdout) -> Result<(), String> {
    let typing_duration = Duration::from_secs(60);
    let mut user_input = String::new();
    let correct_sentence = &get_sentence::sentence();
    let mut cursor_index = 0;

    helpers::render_border(stdout, true)?;
    helpers::print_countdown(
        stdout,
        typing_duration.as_secs_f64(),
        1,
        correct_sentence,
        helpers::USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed),
        helpers::LEFT_OFFSET as u16,
        6,
    )?;
    // if during countdown user typed something, it gets queued into event:poll(). While there's something, clearing it.
    while event::poll(Duration::from_millis(0)).map_err(|e| e.to_string())? {
        let _ = event::read().map_err(|e| e.to_string())?;
    }

    let race_started_at = Instant::now();

    loop {
        if race_started_at.elapsed() >= typing_duration {
            break;
        }
        render(
            stdout,
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
            Event::Key(key_event) => {
                if helpers::is_exit_event(Event::Key(key_event)) {
                    move_to_end_before_exit(stdout)?;
                    return Ok(());
                }
                match key_event.code {
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
                    KeyCode::Left => cursor_index = cursor_index.saturating_sub(1),
                    KeyCode::Right => {
                        if cursor_index < user_input.len() {
                            cursor_index += 1;
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(w, h) => {
                helpers::USABLE_TERMINAL_WIDTH.store(w - 4, Ordering::Relaxed);
                helpers::USABLE_TERMINAL_HEIGHT.store(h - 2, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    let (wpm, accuracy) = statistics(correct_sentence, &user_input.to_string(), typing_duration)?;

    statistics::print_after_race_statistics(wpm, accuracy, stdout)?;
    statistics::save_statistics(wpm, accuracy)?;

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
    let usable_terminal_width = helpers::USABLE_TERMINAL_WIDTH.load(Ordering::Relaxed);
    let elapsed_seconds = race_started_at.elapsed().as_secs_f64();
    // queue!(stdout, Clear(FromCursorDown)).map_err(|e| e.to_string())?;
    queue!(stdout, cursor::Hide).map_err(|e| e.to_string())?;

    helpers::queue_centered_message(
        format!("Elapsed: {:.1}s / {:.1}s", elapsed_seconds, total_seconds).as_str(),
        stdout,
        // leaving space, so + 1.
        first_free_row + 1,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )?;

    let (cursor_col, cursor_row) = cursor_position(
        cursor_index,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    render_user_typing(
        stdout,
        user_input,
        correct_sentence,
        input_start_col,
        input_start_row,
        usable_terminal_width,
    )?;

    queue!(stdout, MoveTo(cursor_col, cursor_row), cursor::Show).map_err(|e| e.to_string())?;

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
            SetForegroundColor(helpers::CUSTOM_FOREGROUND_COLOR_DARK),
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
                    helpers::CUSTOM_FOREGROUND_COLOR
                } else {
                    helpers::CUSTOM_ERROR_COLOR
                }),
                Print(*inputting_char as char)
            )
            .map_err(|e| e.to_string())?;
        }
    }

    queue!(stdout, MoveTo(2, 5), SetForegroundColor(Color::Reset)).map_err(|e| e.to_string())?;

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
