use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::atomic::Ordering::Relaxed,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::terminal;

use crate::helpers::{
    self, CUSTOM_FOREGROUND_COLOR, USABLE_TERMINAL_HEIGHT, format_timestamp,
    move_to_end_before_exit, queue_centered_message,
};

#[derive(Serialize, Deserialize, Clone)]
struct SavedStatistics {
    timestamp: u64,
    wpm: f64,
    accuracy: f64,
}

pub fn get_saved_statistics_json_path() -> Result<PathBuf, String> {
    let proj_dirs: ProjectDirs =
        ProjectDirs::from("speed", "jrm", "heyo").ok_or("Couldnt find data dir")?;
    let save_path = proj_dirs.data_dir();
    return Ok(save_path.join("statistics.json"));
}

pub fn statistics_display_from_args(stdout: &mut std::io::Stdout) -> Result<(), String> {
    helpers::render_border_over_duration(stdout, Duration::from_secs(1), true)?;

    let statistics_path = get_saved_statistics_json_path()?;
    let mut statistics = Vec::new();
    if statistics_path.exists() {
        let file_contents = fs::read_to_string(&statistics_path).map_err(|e| e.to_string())?;
        statistics = serde_json::from_str::<Vec<SavedStatistics>>(&file_contents)
            .map_err(|e| e.to_string())?
    }

    if statistics.is_empty() {
        queue_centered_message(
            "No previous runs found.",
            stdout,
            2,
            Some(CUSTOM_FOREGROUND_COLOR),
            None,
            true,
        )?;

        stdout.flush().map_err(|e| e.to_string())?;
        sleep(Duration::from_secs(5));
        move_to_end_before_exit(stdout)?;
        return Ok(());
    };

    let time = format_timestamp(statistics.last().unwrap().timestamp)?;
    queue_centered_message(
        format!(
            "Last run on {}: WPM {:.1}, accuracy {:.1}%.",
            time,
            statistics.last().unwrap().wpm,
            statistics.last().unwrap().accuracy
        )
        .as_str(),
        stdout,
        2,
        Some(CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )?;

    if statistics.len() == 1 {
        move_to_end_before_exit(stdout)?;
        return Ok(());
    }

    let mut statistics_x4 = statistics.clone();
    // statistics_x4.extend_from_slice(&statistics);
    // statistics_x4.extend_from_slice(&statistics);
    // statistics_x4.extend_from_slice(&statistics);
    // statistics_x4.extend_from_slice(&statistics);

    for (index, single_stat) in statistics_x4.iter().rev().skip(1).enumerate() {
        if index < USABLE_TERMINAL_HEIGHT.load(Relaxed) as usize - 4 {
            let time = format_timestamp(single_stat.timestamp)?;
            queue_centered_message(
                format!(
                    "Run on {}: WPM {:.1}, accuracy {:.1}%.",
                    time, single_stat.wpm, single_stat.accuracy
                )
                .as_str(),
                stdout,
                3 + index as u16,
                Some(CUSTOM_FOREGROUND_COLOR),
                None,
                true,
            )?;
        } else {
            let path = statistics_path
                .to_str()
                .ok_or("Couldn't find statistics path")?;
            let url = format!("file://{}", path.replace(" ", "%20"));
            // problem - link is large, and to make it clickable its even larger. queue_centered_message chunks by lines, and terminal_link is multiple lines.
            // need custom function for this, or not display the path here.
            let terminal_link = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, "here");

            queue_centered_message(
                "...",
                stdout,
                USABLE_TERMINAL_HEIGHT.load(Relaxed) - 1,
                Some(CUSTOM_FOREGROUND_COLOR),
                None,
                true,
            )?;
            queue_centered_message(
                format!("All stats visible {}", terminal_link).as_str(),
                stdout,
                USABLE_TERMINAL_HEIGHT.load(Relaxed),
                Some(CUSTOM_FOREGROUND_COLOR),
                None,
                true,
            )?;
            break;
        }
    }

    stdout.flush().map_err(|e| e.to_string())?;
    sleep(Duration::from_secs(5));
    move_to_end_before_exit(stdout)?;

    return Ok(());
}

pub fn print_after_race_statistics(
    wpm: f64,
    accuracy: f64,
    stdout: &mut std::io::Stdout,
) -> Result<(), String> {
    let (_, terminal_height) = terminal::size().map_err(|e| e.to_string())?;

    helpers::queue_centered_message(
        "Finished!",
        stdout,
        terminal_height - 4,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        false,
    )
    .map_err(|e| e.to_string())?;
    helpers::queue_centered_message(
        format!("WPM: {:.1}, accuracy: {:.1}%.", wpm, accuracy).as_str(),
        stdout,
        terminal_height - 3,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )
    .map_err(|e| e.to_string())?;
    helpers::queue_centered_message(
        "Stats updated. Check with 'heyo stats'.",
        stdout,
        terminal_height - 2,
        Some(helpers::CUSTOM_FOREGROUND_COLOR),
        None,
        true,
    )
    .map_err(|e| e.to_string())?;

    move_to_end_before_exit(stdout)?;

    stdout.flush().map_err(|e| e.to_string())?;

    return Ok(());
}

pub fn save_statistics(wpm: f64, accuracy: f64) -> Result<(), String> {
    let proj_dirs: ProjectDirs =
        ProjectDirs::from("speed", "jrm", "heyo").ok_or("Couldnt find data dir")?;

    let save_path = proj_dirs.data_dir();
    fs::create_dir_all(save_path).map_err(|e| e.to_string())?;

    let statistics_path = save_path.join("statistics.json");

    let mut data_from_file = if statistics_path.exists() {
        //
        let file_contents = fs::read_to_string(&statistics_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<Vec<SavedStatistics>>(&file_contents).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    data_from_file.push(SavedStatistics {
        timestamp,
        wpm,
        accuracy,
    });

    let serialized_struct =
        serde_json::to_string_pretty(&data_from_file).map_err(|e| e.to_string())?;

    fs::write(statistics_path, serialized_struct).map_err(|e| e.to_string())?;

    return Ok(());
}
