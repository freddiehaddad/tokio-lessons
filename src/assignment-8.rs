// Assignment 8: Custom Mini-Runtime
//
// Objective: Build a tiny single-threaded executor that can poll futures to
// completion — no Tokio, no async runtime. Just you, std::future::Future,
// std::task::Waker, and Poll.
//
// This is NOT a Tokio assignment. It's about understanding what Tokio does
// under the hood. You'll use only std — no tokio dependency.
//
// Requirements:
//
//  1. Build a MiniRuntime struct with:
//     - fn new() -> Self
//     - fn spawn(&mut self, future: impl Future<Output = ()> + 'static) — adds
//       a future to the runtime's task queue
//     - fn run(&mut self) — polls all tasks in a loop until all are complete
//  2. Implement a working Waker:
//     - When a future returns Poll::Pending, the runtime needs to know when to
//       poll it again
//     - Build a Waker that signals "this task is ready to be polled" by setting
//       a flag (e.g., an Arc<AtomicBool>)
//     - The runtime's run loop checks which tasks are flagged as ready and
//       polls them
//  3. Implement a custom Sleep future (no tokio!):
//     - TimerFuture::new(duration: Duration) — a future that completes after
//       duration
//     - Spawns a real std::thread that sleeps for the duration, then calls
//       waker.wake() to signal completion
//     - Returns Poll::Pending until the thread signals it's done
//  4. Demonstrate it works in main:
//     fn main() {
//         let mut rt = MiniRuntime::new();
//         rt.spawn(async {
//             println!("Task 1: starting");
//             TimerFuture::new(Duration::from_secs(1)).await;
//             println!("Task 1: done after 1s");
//         });
//         rt.spawn(async {
//             println!("Task 2: starting");
//             TimerFuture::new(Duration::from_millis(500)).await;
//             println!("Task 2: done after 500ms");
//         });
//         rt.run(); // blocks until all tasks complete
//     }
//
//     Expected output: both tasks start immediately, Task 2 finishes first.
//  5. Write tests:
//     - A single task completes
//     - Multiple tasks run concurrently (Task 2 with shorter sleep finishes
//       before Task 1)
//     - A task with zero-duration sleep completes immediately
//
// Key concepts you need to understand:
// ┌───────────────────────────┬───────────────────────────────────────────────┐
// │ Concept                   │ Role                                          │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ Future::poll(             │ The runtime calls this to drive a future      │
// │  self: Pin<&mut Self>,    │ forward                                       │
// │  cx: &mut Context)        │                                               │
// │ )                         │                                               │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ Poll::Ready(val)          │ Future is done, here's the result             │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ Poll::Pending             │ Future isn't done yet, call waker.wake()      │
// │                           │ when it might be                              │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ Waker                     │ Handle that the future uses to tell the       │
// │                           │ runtime "poll me again"                       │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ RawWaker + RawWakerVTable │ Low-level primitives to build a custom Waker  │
// ├───────────────────────────┼───────────────────────────────────────────────┤
// │ Pin<Box<dyn Future>>      │ How to store a future that must not move in   │
// │                           │ memory                                        │
// └───────────────────────────┴───────────────────────────────────────────────┘
//
// Hints:
//
//  - Store tasks as Vec<Pin<Box<dyn Future<Output = ()>>>>
//  - For each task, pair it with an Arc<AtomicBool> flag (the "wake" signal)
//  - Build a Waker from a RawWaker: you need a RawWakerVTable with clone, wake,
//    wake_by_ref, and drop functions. The wake function sets the AtomicBool to
//    true
//  - The run loop: iterate tasks, skip ones not flagged as ready, poll ready
//    ones, remove completed ones. Loop until empty.
//  - Initially flag all tasks as ready (they need at least one poll to start)
//  - TimerFuture stores an Arc<AtomicBool> for completion status. Its poll
//    checks the flag; if not set, it stashes the waker and returns Pending. The
//    background thread sets the flag and calls wake().
//
// Grading criteria:
//
//  - No async runtime dependency (no tokio, no async-std)
//  - Custom Waker built from RawWaker/RawWakerVTable
//  - TimerFuture correctly implements Future with a background thread
//  - Multiple futures run concurrently on a single thread
//  - Tests verify correct behavior
//
// Why this matters: Every .await you've written in Assignments 1-7 was driven
// by machinery exactly like this. Understanding it demystifies async Rust
// completely.
//
// This is the capstone. Take your time — and enjoy it!

#[tokio::main]
async fn main() {
    todo!("Implement Assignment 8 Solution");
}
