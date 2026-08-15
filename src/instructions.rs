use std::{
    io::{Stdout, Write},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType::FromCursorDown},
};

use crate::helpers;

pub fn run(stdout: &mut Stdout) -> Result<(), String> {
    execute!(stdout, MoveTo(1, 1), Clear(FromCursorDown),).map_err(|e| e.to_string())?;

    helpers::render_border(stdout)?;
    stdout.flush().map_err(|e| e.to_string())?;
    sleep(Duration::from_secs(5));

    return Ok(());
}
