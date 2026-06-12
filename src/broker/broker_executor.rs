use std::collections::VecDeque;

#[derive(Debug)]
pub struct Job {
    pub payload: String,
}

#[derive(Debug)]
pub struct BrokerState {
    pub queued: VecDeque<Job>,
}

impl BrokerState {
    pub fn new() -> BrokerState {
        BrokerState {
            queued: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, payload: String) -> &Self {
        self.queued.push_back(Job { payload });
        self
    }

    pub fn dequeue(&mut self) -> Option<Job> {
        self.queued.pop_front()
    }

    pub fn list(&self) {
        println!("{:?}", self)
    }
}
