use std::env;
mod host;
mod invitee;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();

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
