# Message Broker

A lightweight message broker written in Rust.

The project is being built incrementally as a learning exercise in message queue design, with each release introducing additional broker capabilities.

## Features

- Interactive REPL
- FIFO in-memory queue
- Job IDs
- Processing state
- Job acknowledgement
- Failure and retry handling
- Dead-letter queue
- Queue inspection

## Running

```bash
cargo run
```

## Commands

### Enqueue a job

```
enqueue <payload>
```

Creates a new job and places it at the back of the queue.

Example:

```
enqueue send_push_notification
```

---

### Dequeue a job

```
dequeue
```

Removes the next job from the queue and moves it into the processing state.

---

### Acknowledge a job

```
ack <job_id>
```

Marks a processing job as successfully completed and removes it from the broker.

Example:

```
ack job-1
```

---

### Fail a job

```
fail <job_id>
```

Marks a processing job as failed.

If the retry limit has not been reached, the job is placed at the back of the queue.

If the maximum retry count has been reached, the job is moved to the dead-letter queue.

Example:

```
fail job-1
```

---

### List broker state

```
list
```

Displays:

- Queued jobs
- Processing jobs
- Dead-letter jobs

along with each job's retry count.

---

### Exit

```
exit
```

Terminates the application.

## Example Workflow

```
enqueue send_push_notification
dequeue
fail job-1
dequeue
ack job-1
```

## Current Limitations

- In-memory storage only
- No persistence
- Single-process execution
- No worker pool
- No networking/API layer

## Roadmap

- Persistent storage
- Scheduled jobs
- Multiple concurrent workers
- TCP/HTTP interface

## Possible Future Ideas

- Multiple named queues
- Job priorities
- Visibility timeouts
- Persistent storage backends
- Pub/Sub support
- Metrics and observability
- Distributed workers
- Web dashboard
