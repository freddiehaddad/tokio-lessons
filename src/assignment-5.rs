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

#[tokio::main]
async fn main() {
    todo!("Implement assignment 5");
}
