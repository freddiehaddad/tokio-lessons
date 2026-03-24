// Assignment 5: Producer-Consumer Pipeline
//
// Objective: Build a multi-stage async data pipeline where data flows through
// distinct stages connected by channels, with backpressure.
//
// Scenario: You're building a log processing pipeline:
//
//  1. Producer — generates raw log entries
//  2. Workers — parse and transform each log entry
//  3. Collector — aggregates and summarizes results
//
// Requirements:
//
//  1. Producer stage: Generate 100 simulated log entries (e.g.,
//     "2026-03-21T12:00:00 INFO User logged in", "2026-03-21T12:00:01 ERROR
//     Database timeout", etc.). Mix of INFO, WARN, and ERROR levels. Send them
//     into a bounded mpsc channel (capacity 10).
//  2. Worker stage: Spawn 3 worker tasks that read from the producer channel
//     concurrently. Each worker should:
//     - Parse the log level (INFO/WARN/ERROR) from the entry
//     - Simulate processing time with a short sleep (50–150ms)
//     - Send a processed result (a struct with level, message, worker_id) into
//       a second bounded mpsc channel
//  3. Collector stage: A single task that receives all processed results and
//     produces a final summary:
//     - Count of logs per level (INFO: X, WARN: Y, ERROR: Z)
//     - Which worker processed the most entries
//     - Total processing time
//  4. Use bounded channels (capacity 10) between each stage — this creates
//     natural backpressure (the producer slows down if workers are busy,
//     workers slow down if the collector is busy)
//  5. Print the final summary at the end
//
// Key concepts to practice:
//
//  - Multi-producer patterns (3 workers sending to 1 collector)
//  - Bounded channels for backpressure
//  - Clean shutdown via channel closure (drop all senders → receiver returns
//    None)
//  - Coordinating multiple pipeline stages
//
// Hints:
//
//  - The producer channel needs a multi-consumer pattern — but mpsc is
//    multi-producer, single-consumer. For sharing one receiver among 3 workers,
//    wrap it in Arc<Mutex<Receiver<T>>> or use an async-channel crate which
//    supports multi-consumer. Alternatively, the producer can round-robin
//    distribute to 3 separate channels.
//  - Drop all tx clones when done to signal downstream stages to shut down
//  - The collector can use a HashMap to aggregate counts
//
// Grading criteria:
//
//  - Three distinct pipeline stages, connected by channels
//  - Bounded channels providing backpressure
//  - Multiple workers processing concurrently
//  - Clean shutdown via channel closure (no cancellation tokens needed here)
//  - Correct final summary

use chrono::{DateTime, Local};
use rand::prelude::*;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::Mutex,
    sync::mpsc::{Receiver, Sender, channel},
    task::spawn,
};

const LOG_ENTRIES: usize = 100;
const CHANNEL_CAPACITY: usize = 10;
const NUM_WORKERS: u8 = 3;

// Simulated event level
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

// Simulated event
struct Event {
    level: LogLevel,
    timestamp: DateTime<Local>,
    event: String,
}

// Processed event
struct Job {
    timestamp: DateTime<Local>,
    level: LogLevel,
    worker: u8,
    message: String,
}

async fn start_producer(tx: Sender<Event>) {
    const LOG_LEVELS: &[LogLevel] =
        &[LogLevel::Info, LogLevel::Warn, LogLevel::Error];
    const EVENT_TYPES: &[&str] = &[
        "Database timeout",
        "User logged in",
        "Connection reset",
        "Account updated",
    ];

    for event_id in 0..LOG_ENTRIES {
        // Pick a random log level
        let level = LOG_LEVELS.choose(&mut rand::rng()).unwrap();
        let message = EVENT_TYPES.choose(&mut rand::rng()).unwrap();
        let timestamp = Local::now();

        // Generate log entries and send them
        let event = Event {
            level: *level,
            timestamp,
            event: message.to_string(),
        };
        println!(
            "EVENT: {} {:?} {}",
            event.timestamp, event.level, event.event
        );
        if let Err(e) = tx.send(event).await {
            eprintln!("Failed to generate event {event_id}: {e}");
        };
    }
}

async fn process_event(id: u8, event: Event) -> Job {
    let sleep_ms = match event.level {
        LogLevel::Info => 50,
        LogLevel::Warn => 100,
        LogLevel::Error => 150,
    };
    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;

    let job = Job {
        timestamp: Local::now(),
        worker: id,
        level: event.level,
        message: event.event,
    };

    println!(
        "WORKER({}): processed {:?} event for {} at {}",
        job.worker, job.level, job.message, job.timestamp
    );

    job
}

async fn start_worker(
    id: u8,
    rx: Arc<Mutex<Receiver<Event>>>,
    tx: Sender<Job>,
) {
    while let Some(event) = rx.lock().await.recv().await {
        let job = process_event(id, event).await;
        if let Err(e) = tx.send(job).await {
            eprintln!("Worker {id} failed to send processed job: {e}");
        }
    }
}

#[tokio::main]
async fn main() {
    let (producer_tx, producer_rx) = channel(CHANNEL_CAPACITY);
    let (collector_tx, mut collector_rx) = channel(CHANNEL_CAPACITY);
    let mut log_count = HashMap::from([
        (LogLevel::Info, 0),
        (LogLevel::Warn, 0),
        (LogLevel::Error, 0),
    ]);
    let mut worker_stats: HashMap<u8, u16> =
        (0..NUM_WORKERS).map(|id| (id, 0_u16)).collect();
    let start_time = Instant::now();

    // Start the producer
    spawn(async move {
        let _ = start_producer(producer_tx).await;
    });

    // Start the workers
    let producer_shared = Arc::new(Mutex::new(producer_rx));
    for worker in 0..NUM_WORKERS {
        let producer_clone = producer_shared.clone();
        let collector_clone = collector_tx.clone();
        spawn(async move {
            let _ = start_worker(worker, producer_clone, collector_clone).await;
        });
    }
    // Drop the channel to signal no more work
    drop(collector_tx);

    // Collect the results
    while let Some(job) = collector_rx.recv().await {
        log_count
            .entry(job.level)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        worker_stats
            .entry(job.worker)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    let elapsed = start_time.elapsed();

    println!("  Total time: {elapsed:02?}");
    println!("   Log count: {log_count:?}");
    println!("Worker stats: {worker_stats:?}");
    if let Some(busiest_worker) = worker_stats.iter().max_by_key(|v| v.1) {
        println!("Busiest worker: {}: {}", busiest_worker.0, busiest_worker.1);
    };
}
