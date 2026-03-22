// Assignment 2: Rate-Limited Task Queue
//
// Objective: Process a batch of work items concurrently, but never exceed N
// tasks running at the same time.
//
// Requirements:
//
//  1. Define ~20 simulated work items (each sleeps for a random 100–500ms and
//     returns a result, e.g. format!("Task {id} completed"))
//  2. Process all items concurrently, but at most N tasks may run
//     simultaneously
//     - use a const MAX_CONCURRENT: usize = 5
//  3. Use tokio::sync::Semaphore to enforce the limit
//  4. For each task, print:
//     - When it starts (with its task ID)
//     - When it finishes (with its task ID and how long it took)
//  5. Print total elapsed time at the end
//  6. Bonus: When a task starts, also print how many tasks are currently
//     in-flight (hint: MAX_CONCURRENT - semaphore.available_permits())
//
// Why this matters: In Assignment 1 you fired off all requests at once. In real
// systems - API rate limits, database connection pools, file descriptor limits
// - you need bounded concurrency. Semaphore is the standard async primitive for
// this.
//
// Hints:
//
//  - Arc<Semaphore> to share across spawned tasks
//  - let permit = semaphore.acquire().await.unwrap() — the slot is held until
//    permit is dropped
//  - Add rand to your Cargo.toml for random sleep durations
//  - tokio::time::sleep for the simulated work
//
// Grading criteria:
//
//  - You can observe from the output that at most 5 tasks are running at once
//  - Semaphore is used correctly (acquire before work, release via drop after)
//  - Errors are handled, not unwrapped carelessly
//  - Clean, readable code
use rand::random_range;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::sync::mpsc;

const MAX_CONCURRENT: usize = 5;
const NUM_TASKS: usize = 20;
const MIN_SLEEP_MS: u64 = 100;
const MAX_SLEEP_MS: u64 = 500;

#[tokio::main]
async fn main() {
    // Record the start time of the entire operation
    let start_time = Instant::now();

    // Create an mpsc channel to receive results from tasks
    let (tx, mut rx) = mpsc::channel(NUM_TASKS);

    // Create a semaphore with MAX_CONCURRENT permits
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));

    // Vector to hold the JoinHandles of the spawned tasks
    let mut handles = Vec::new();

    // Spawn a task to collect and print results from the channel
    handles.push(tokio::spawn(async move {
        // Collect and print results as they arrive
        loop {
            let Some(result) = rx.recv().await else {
                break;
            };
            println!("{}", result);
        }
        Ok::<String, ()>("Collector task completed".to_string())
    }));

    // Simulate 20 work items
    for id in 0..NUM_TASKS {
        let tx = tx.clone();
        let semaphore = semaphore.clone();

        // Spawn a task for each work item
        handles.push(tokio::spawn(async move {
            // Attempt to acquire a permit from the semaphore before spawning
            // the task
            let Ok(_permit) = semaphore.acquire().await else {
                eprintln!("Task {} failed to acquire semaphore permit", id);
                return Err(());
            };

            // Record the start time of the task
            let start_time = Instant::now();

            // Send a message when the task starts, including how many tasks are
            // currently in-flight
            tx.send(format!(
                "Task {} started at {:?} (Currently in-flight: {})",
                id,
                start_time,
                MAX_CONCURRENT - semaphore.available_permits()
            ))
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "Failed to send start message for task {}: {:?}",
                    id, e
                );
            });

            // Simulate work by sleeping for a random duration between 100 and
            // 500 ms
            tokio::time::sleep(Duration::from_millis(random_range(
                MIN_SLEEP_MS..=MAX_SLEEP_MS,
            )))
            .await;

            // Record the elapsed time for the task
            let elapsed = start_time.elapsed();
            tx.send(format!("Task {} completed in {:.2?}", id, elapsed,))
                .await
                .unwrap_or_else(|e| {
                    eprintln!(
                        "Failed to send completion message for task {}: {:?}",
                        id, e
                    );
                });

            let result = format!("Task {} completed", id);
            Ok::<String, ()>(result)
        }))
    }

    // Close the sender to signal that no more messages will be sent
    drop(tx);

    // Wait for all tasks to complete
    for handle in handles {
        let Ok(result) = handle.await else {
            eprintln!("Task panicked");
            continue;
        };

        let Ok(_result) = result else {
            eprintln!("Task returned an error");
            continue;
        };

        // println!("Result from task: {}", result);
    }

    // Print total elapsed time
    let elapsed = start_time.elapsed();
    println!("Total elapsed time: {:.2?}", elapsed);
}
