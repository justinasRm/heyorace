use std::{
    env,
    io::{self},
};

use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};

use crate::helpers::set_initial_terminal_dimensions;
mod get_sentence;
mod helpers;
mod host;
mod instructions;
mod invitee;

const MIN_TERMINAL_WIDTH: u16 = 110;
const MIN_TERMINAL_HEIGHT: u16 = 20;

pub const CLEAN_EXIT_EVENT_MESSAGE: &str = "cleanexit";

fn clean_exit(e: String) -> Result<(), String> {
    {
        if e != CLEAN_EXIT_EVENT_MESSAGE {
            return Err(e.to_string());
        }
        return Ok(());
    }
}

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Result<Self, String> {
        enable_raw_mode().map_err(|e| e.to_string())?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

fn main() -> Result<(), String> {
    let _raw_mode = RawModeGuard::new().map_err(|e| e.to_string())?;

    let mut stdout = io::stdout();

    let (terminal_width, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    set_initial_terminal_dimensions()?;

    if terminal_width < MIN_TERMINAL_WIDTH || terminal_height < MIN_TERMINAL_HEIGHT {
        return Err(format!(
            "Terminal is too small. Minimum size is {}x{}, current size is {}x{}.",
            MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT, terminal_width, terminal_height
        ));
    }

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => match instructions::run(&mut stdout) {
            Ok(()) => match host::solo_typing_speed_test(&mut stdout) {
                Ok(()) => Ok(()),
                Err(e) => clean_exit(e),
            },
            Err(e) => clean_exit(e),
        },
        [cmd] if cmd == "type" => match host::solo_typing_speed_test(&mut stdout) {
            Ok(()) => Ok(()),
            Err(e) => clean_exit(e),
        },
        [cmd] if cmd == "debug" => match helpers::debug() {
            Ok(()) => Ok(()),
            Err(e) => clean_exit(e),
        },
        // [cmd, code] if cmd == "join" => match invitee::join_race(code) {
        //     Ok(()) => Ok(()),
        //     Err(e) => Err(e.to_string()),
        // },
        [..] => Err("wrong cli command. use 'heyorace'.".to_string()),
    }
}
