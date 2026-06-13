use std::{
    collections::{HashMap, VecDeque},
    io::BufRead,
};

use crate::storage::storage::{record_event, recover_events};

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub payload: String,
    pub retries: u32,
    pub max_retries: u32,
}

#[derive(Debug)]
pub struct BrokerState {
    pub queued: VecDeque<Job>,
    pub processing: HashMap<String, Job>,
    pub dead_letter: HashMap<String, Job>,
    pub next_id: u64,
    pub default_max_retries: u32,
}

#[derive(Debug)]
enum LogEventCommand {
    Enqueue { job: Job },
    Dequeue,
    Ack { job_id: String },
    Fail { job_id: String },
}

impl BrokerState {
    pub fn new() -> BrokerState {
        BrokerState {
            queued: VecDeque::new(),
            processing: HashMap::new(),
            dead_letter: HashMap::new(),
            next_id: 1,
            default_max_retries: 3,
        }
    }

    pub fn enqueue(&mut self, payload: String) -> Result<bool, String> {
        let new_job = Job {
            id: format!("job-{}", self.next_id),
            payload,
            retries: 0,
            max_retries: self.default_max_retries,
        };

        if let Err(error) = record_event(format!(
            "enqueue {} {} {} {}\n",
            new_job.id, new_job.payload, new_job.retries, new_job.max_retries
        )) {
            return Err(error);
        }

        Ok(self.apply_enqueue(new_job))
    }

    fn apply_enqueue(&mut self, job: Job) -> bool {
        self.queued.push_back(job);

        self.next_id += 1;
        true
    }

    pub fn dequeue(&mut self) -> Result<Option<String>, String> {
        // self.queued.pop_front().inspect(|job| {
        //     self.processing.insert(job.id.to_string(), job.clone());
        // })

        let Some(job) = self.queued.get(0) else {
            return Ok(None);
        };

        if let Err(error) = record_event(format!("dequeue {}\n", job.id)) {
            return Err(error);
        }

        Ok(self.apply_dequeue())
    }

    fn apply_dequeue(&mut self) -> Option<String> {
        if let Some(dequeued_job) = self.queued.pop_front() {
            let job_id = dequeued_job.id.to_string();

            self.processing
                .insert(dequeued_job.id.to_string(), dequeued_job);

            Some(job_id)
        } else {
            None
        }
    }

    pub fn ack(&mut self, job_id: String) -> Result<bool, String> {
        let Some(_) = self.processing.get(&job_id) else {
            return Ok(false);
        };

        if let Err(error) = record_event(format!("ack {}\n", job_id)) {
            return Err(error);
        }

        Ok(self.apply_ack(job_id))
    }

    fn apply_ack(&mut self, job_id: String) -> bool {
        match self.processing.remove(&job_id) {
            Some(_) => return true,
            None => return false,
        }
    }

    pub fn fail(&mut self, job_id: String) -> Result<Option<bool>, String> {
        let Some(_) = self.processing.get(&job_id) else {
            return Ok(Some(false));
        };

        if let Err(error) = record_event(format!("fail {}\n", job_id)) {
            return Err(error);
        }

        Ok(self.apply_fail(job_id))
    }

    fn apply_fail(&mut self, job_id: String) -> Option<bool> {
        let Some(failed_job) = self.processing.remove(&job_id) else {
            return None;
        };

        if failed_job.retries >= failed_job.max_retries {
            self.dead_letter
                .insert(failed_job.id.to_string(), failed_job);
            return Some(false);
        }

        self.queued.push_back(Job {
            retries: failed_job.retries + 1,
            ..failed_job
        });

        Some(true)
    }

    pub fn list(&self) {
        println!("================ MESSAGE BROKER ================ \n");

        println!("Queued Jobs ({})", self.queued.len());
        println!("-----------------------------------------------");
        if self.queued.is_empty() {
            println!("(empty)");
        } else {
            for single_job in &self.queued {
                println!(
                    "- {} {:?} attempts: {}/{}",
                    single_job.id, single_job.payload, single_job.retries, single_job.max_retries
                );
            }
        }

        println!("\n");

        println!("Processing Jobs ({})", self.processing.len());
        println!("-----------------------------------------------");
        if self.processing.is_empty() {
            println!("(empty)");
        } else {
            for (key, value) in &self.processing {
                println!(
                    "- {} {:?} attempts: {}/{}",
                    key, value.payload, value.retries, value.max_retries
                );
            }
        }

        println!("\n");

        println!("Dead Letter Queue ({})", self.dead_letter.len());
        println!("-----------------------------------------------");
        if self.dead_letter.is_empty() {
            println!("(empty)");
        } else {
            for (key, value) in &self.dead_letter {
                println!(
                    "- {} {:?} attempts: {}/{}",
                    key, value.payload, value.retries, value.max_retries
                );
            }
        }
    }

    pub fn replay(&mut self) {
        let Some(event_log_file) = recover_events() else {
            return ();
        };

        for single_line in event_log_file.lines().flatten() {
            match parse_event_log_command(&single_line) {
                Ok(LogEventCommand::Enqueue { job }) => {
                    self.apply_enqueue(job);
                }
                Ok(LogEventCommand::Dequeue) => {
                    self.apply_dequeue();
                }
                Ok(LogEventCommand::Ack { job_id }) => {
                    self.apply_ack(job_id);
                }
                Ok(LogEventCommand::Fail { job_id }) => {
                    self.apply_fail(job_id);
                }
                _ => {
                    continue;
                }
            }
        }
    }
}

fn parse_event_log_command(command_log: &str) -> Result<LogEventCommand, String> {
    let split_full_command: Vec<&str> = command_log.split_whitespace().collect();

    if split_full_command.is_empty() {
        return Err(format!("Empty command: {}", command_log));
    }

    let Some(main_command) = split_full_command.get(0).copied() else {
        return Err(format!("Command cannot be empty"));
    };

    let event_cmd = match main_command {
        "enqueue" => {
            if split_full_command.len() != 5 {
                return Err(format!("Invalid enqueue command {}", command_log));
            }

            let Some(job_id) = split_full_command.get(1).copied() else {
                return Err(format!("Missing job_id from enqueue command"));
            };

            let Some(payload) = split_full_command.get(2).copied() else {
                return Err(format!("Missing payload from enqueue command"));
            };

            let Some(retries) = split_full_command
                .get(3)
                .copied()
                .and_then(|retry| retry.parse::<u32>().ok())
            else {
                return Err(format!("Invalid retries type on enqueue command"));
            };

            let Some(max_retries) = split_full_command
                .get(4)
                .copied()
                .and_then(|max_retry| max_retry.parse::<u32>().ok())
            else {
                return Err(format!("Invalid max_retries type on enqueue command"));
            };

            LogEventCommand::Enqueue {
                job: Job {
                    id: job_id.to_string(),
                    payload: payload.to_string(),
                    retries,
                    max_retries,
                },
            }
        }
        "dequeue" => {
            if split_full_command.len() > 2 {
                return Err(format!("Invalid dequeue command {}", command_log));
            }

            LogEventCommand::Dequeue
        }
        "ack" => {
            if split_full_command.len() != 2 {
                return Err(format!("Invalid ack command {}", command_log));
            }

            let Some(job_id) = split_full_command.get(1).copied() else {
                return Err(format!("Missing job_id from ack command"));
            };

            LogEventCommand::Ack {
                job_id: job_id.to_string(),
            }
        }
        "fail" => {
            if split_full_command.len() != 2 {
                return Err(format!("Invalid fail command {}", command_log));
            }

            let Some(job_id) = split_full_command.get(1).copied() else {
                return Err(format!("Missing job_id from fail command"));
            };

            LogEventCommand::Fail {
                job_id: job_id.to_string(),
            }
        }
        _ => {
            return Err(format!("Invalid command {}", command_log));
        }
    };

    Ok(event_cmd)
}
