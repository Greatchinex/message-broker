use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub payload: String,
    pub attempts: u64,
    pub max_attempts: u64,
}

#[derive(Debug)]
pub struct BrokerState {
    pub queued: VecDeque<Job>,
    pub processing: HashMap<String, Job>,
    pub next_id: u64,
    pub default_max_attempts: u64,
}

impl BrokerState {
    pub fn new() -> BrokerState {
        BrokerState {
            queued: VecDeque::new(),
            processing: HashMap::new(),
            next_id: 1,
            default_max_attempts: 3,
        }
    }

    pub fn enqueue(&mut self, payload: String) -> &Self {
        self.queued.push_back(Job {
            id: format!("job-{}", self.next_id),
            payload,
            attempts: 0,
            max_attempts: self.default_max_attempts,
        });

        self.next_id = self.next_id + 1;
        self
    }

    pub fn dequeue(&mut self) -> Option<String> {
        // self.queued.pop_front().inspect(|job| {
        //     self.processing.insert(job.id.to_string(), job.clone());
        // })

        if let Some(dequeued_job) = self.queued.pop_front() {
            let job_id = dequeued_job.id.to_string();
            self.processing
                .insert(dequeued_job.id.to_string(), dequeued_job);

            Some(job_id)
        } else {
            None
        }
    }

    pub fn ack(&mut self, job_id: String) -> bool {
        match self.processing.remove(&job_id) {
            Some(_) => return true,
            None => return false,
        }
    }

    pub fn fail(&mut self, job_id: String) -> Option<bool> {
        let Some(failed_job) = self.processing.remove(&job_id) else {
            return None;
        };

        if failed_job.attempts > failed_job.max_attempts {
            return Some(false);
        }

        self.queued.push_back(Job {
            attempts: failed_job.attempts + 1,
            ..failed_job
        });

        Some(true)
    }

    pub fn list(&self) {
        println!("Queued: \n");
        for single_job in &self.queued {
            println!(
                "- {} {:?} ({}/{})",
                single_job.id, single_job.payload, single_job.attempts, single_job.max_attempts
            );
        }

        println!("\n");

        println!("Processing: \n");
        for (key, value) in &self.processing {
            println!(
                "- {} {:?} ({}/{})",
                key, value.payload, value.attempts, value.max_attempts
            );
        }
    }
}
