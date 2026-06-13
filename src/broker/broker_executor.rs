use std::collections::{HashMap, VecDeque};

use crate::storage::storage::record_event;

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub payload: String,
    pub retries: u64,
    pub max_retries: u64,
}

#[derive(Debug)]
pub struct BrokerState {
    pub queued: VecDeque<Job>,
    pub processing: HashMap<String, Job>,
    pub dead_letter: HashMap<String, Job>,
    pub next_id: u64,
    pub default_max_retries: u64,
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

        self.queued.push_back(new_job);

        self.next_id += 1;
        Ok(true)
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

        if let Some(dequeued_job) = self.queued.pop_front() {
            let job_id = dequeued_job.id.to_string();

            self.processing
                .insert(dequeued_job.id.to_string(), dequeued_job);

            Ok(Some(job_id))
        } else {
            Ok(None)
        }
    }

    pub fn ack(&mut self, job_id: String) -> Result<bool, String> {
        let Some(_) = self.processing.get(&job_id) else {
            return Ok(false);
        };

        if let Err(error) = record_event(format!("ack {}\n", job_id)) {
            return Err(error);
        }

        match self.processing.remove(&job_id) {
            Some(_) => return Ok(true),
            None => return Ok(false),
        }
    }

    pub fn fail(&mut self, job_id: String) -> Result<Option<bool>, String> {
        let Some(_) = self.processing.get(&job_id) else {
            return Ok(Some(false));
        };

        if let Err(error) = record_event(format!("fail {}\n", job_id)) {
            return Err(error);
        }

        let Some(failed_job) = self.processing.remove(&job_id) else {
            return Ok(None);
        };

        if failed_job.retries >= failed_job.max_retries {
            self.dead_letter
                .insert(failed_job.id.to_string(), failed_job);
            return Ok(Some(false));
        }

        self.queued.push_back(Job {
            retries: failed_job.retries + 1,
            ..failed_job
        });

        Ok(Some(true))
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
}
