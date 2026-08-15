use std::{
    env,
    io::{self},
};

use crossterm::terminal;
mod get_sentence;
mod helpers;
mod host;
mod instructions;
mod invitee;

const MIN_TERMINAL_WIDTH: u16 = 110;
const MIN_TERMINAL_HEIGHT: u16 = 20;

fn main() -> Result<(), String> {
    let mut stdout = io::stdout();

    let (terminal_width, terminal_height) = terminal::size().map_err(|e| e.to_string())?;
    if terminal_width < MIN_TERMINAL_WIDTH || terminal_height < MIN_TERMINAL_HEIGHT {
        return Err(format!(
            "Terminal is too small. Minimum size is {}x{}, current size is {}x{}.",
            MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT, terminal_width, terminal_height
        ));
    }

    let args: Vec<String> = env::args().skip(1).collect();

    instructions::run(&mut stdout)?;

    match args.as_slice() {
        [cmd] if cmd == "type" => match host::solo_typing_speed_test(&mut stdout) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        // [cmd, code] if cmd == "join" => match invitee::join_race(code) {
        //     Ok(()) => Ok(()),
        //     Err(e) => Err(e.to_string()),
        // },
        [..] => Err("wrong cli command. use 'heyorace type'.".to_string()),
    }
}
