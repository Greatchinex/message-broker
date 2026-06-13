use crate::broker::broker_executor::BrokerState;
use crate::repl::repl_executor;

pub mod broker;
pub mod repl;
pub mod storage;

fn main() {
    // NOTE: INITIAL IMPLEMENTATION IS A COMMAND LINE REPL. BUT IT WILL BE EXTENDED TO TAKE DIFFERENT SOURCES

    let mut broker = BrokerState::new();

    broker.replay();
    repl_executor::run(&mut broker);
}
