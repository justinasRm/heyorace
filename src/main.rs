use std::env;
mod get_sentence;
mod helpers;
mod host;
mod invitee;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    // TODO: need to enforce minimal terminal height and width.
    // TODO: need an instructions page first.
    match args.as_slice() {
        [cmd] if cmd == "host" => match host::host_race() {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        [cmd, code] if cmd == "join" => match invitee::join_race(code) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
        [..] => Err("Wrong usage".to_string()),
    }
}
