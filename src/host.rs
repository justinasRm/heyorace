use std::{
    io::{Result, Write, stdout},
    print, println,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub fn host_race() -> Result<()> {
    // println!("will be hosting the race from here");
    let _raw_mode = RawModeGuard::new()?;

    let mut user_input = "";
    let final_sentence = "The brown fox jumps over quick fuck idk";

    println!("Your sentence: '{}'\n\r", final_sentence);
    println!("Start whenever you are ready!\n\r");
    // some user input loop, until he finishes the sentence
    loop {
        //
        let event = event::read()?;

        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => {
                    print!("{c}");
                    stdout().flush()?;
                }
                KeyCode::Backspace => {
                    println!("Backspace\r")
                }
                KeyCode::Enter => {
                    println!("Enter")
                }
                KeyCode::Esc => break,
                _ => {}
            },
            _ => {}
        }
    }

    return Ok(());
}
