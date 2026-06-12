use std::{io, process};

#[derive(Debug)]
enum TerminalCommand {
    Enqueue { value: String },
    Dequeue,
    List,
    Exit,
}

pub fn run() {
    loop {
        println!("=====================PLEASE TYPE A COMMAND==============================");
        let mut command_input = String::new();

        match io::stdin().read_line(&mut command_input) {
            Ok(_) => {}
            Err(error) => {
                println!("Failed to read command: {error}");
                process::exit(1);
            }
        }

        let cmd = match parse_command(&command_input) {
            Ok(cmd) => cmd,
            Err(err) => {
                println!("{err}");
                continue;
            }
        };

        match cmd {
            TerminalCommand::Enqueue { value } => {
                println!("Enqueue {value}");
            }
            TerminalCommand::Dequeue => {
                println!("Dequeue");
            }
            TerminalCommand::List => {
                println!("List");
            }
            TerminalCommand::Exit => {
                println!("Exiting program...");
                break;
            }
        }
    }
}

fn parse_command(command_line_input: &str) -> Result<TerminalCommand, String> {
    let split_full_command: Vec<&str> = command_line_input.split_whitespace().collect();

    if split_full_command.is_empty() {
        return Err(format!("Empty command: {}", command_line_input));
    }

    let raw_cmd = match split_full_command.get(0).copied() {
        Some(raw_cmd) => raw_cmd,
        None => {
            return Err(format!("Command cannot be empty"));
        }
    };

    let cmd = match raw_cmd {
        "enqueue" => {
            if split_full_command.len() != 2 {
                return Err(format!("Invalid enqueue command {}", command_line_input));
            }

            let Some(cmd_value) = split_full_command.get(1).copied() else {
                return Err(format!("Value not found for set command"));
            };

            TerminalCommand::Enqueue {
                value: cmd_value.to_string(),
            }
        }
        "dequeue" => {
            if split_full_command.len() != 1 {
                return Err(format!("Invalid dequeue command"));
            }

            TerminalCommand::Dequeue
        }
        "list" => {
            if split_full_command.len() != 1 {
                return Err(format!("Invalid list command"));
            }

            TerminalCommand::List
        }
        "exit" => {
            if split_full_command.len() != 1 {
                return Err(format!("Invalid exit command"));
            }

            TerminalCommand::Exit
        }
        _ => {
            return Err(format!("Invalid command {}", command_line_input));
        }
    };

    Ok(cmd)
}
