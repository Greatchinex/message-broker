use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
};

const LOG_FILE_NAME: &str = "storage.log";

pub fn record_event(action: String) -> Result<(), String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_NAME);

    let mut file = match file {
        Ok(single) => single,
        Err(error) => {
            return Err(format!("Failed to open log file: {}", error));
        }
    };

    match file.write_all(action.as_bytes()) {
        Ok(_) => {
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Error appending command to file: {}", error));
        }
    };
}

pub fn recover_events() -> Option<BufReader<File>> {
    match File::open(LOG_FILE_NAME) {
        Ok(log_file) => {
            return Some(BufReader::new(log_file));
        }
        Err(_) => {
            return None;
        }
    }
}
