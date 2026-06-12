use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub payload: String,
}

#[derive(Debug)]
pub struct BrokerState {
    pub queued: VecDeque<Job>,
    pub processing: HashMap<String, Job>,
    pub next_id: u64,
}

impl BrokerState {
    pub fn new() -> BrokerState {
        BrokerState {
            queued: VecDeque::new(),
            processing: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, payload: String) -> &Self {
        self.queued.push_back(Job {
            id: format!("job-{}", self.next_id),
            payload,
        });

        self.next_id = self.next_id + 1;
        self
    }

    pub fn dequeue(&mut self) -> Option<Job> {
        self.queued.pop_front().inspect(|job| {
            self.processing.insert(job.id.to_string(), job.clone());
        })
    }

    pub fn list(&self) {
        println!("{:?}", self)
    }
}
