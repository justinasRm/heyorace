use std::{io::Stdout, time::Duration};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType::FromCursorDown},
};

use crate::helpers;

pub fn run(stdout: &mut Stdout) -> Result<(), String> {
    execute!(stdout, MoveTo(1, 1), Clear(FromCursorDown)).map_err(|e| e.to_string())?;

    helpers::render_border_over_duration(stdout, Duration::from_secs(1))?;
    return Ok(());
}
