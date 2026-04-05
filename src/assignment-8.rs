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
//  - Store tasks as Vec<Pin<Box<dyn Future<Output = ()>>>>. Pair each with an
//    Arc<AtomicBool> flag (the "wake" signal) - a Task struct works well here.
//  - Build a Waker from a RawWaker: you need a RawWakerVTable with clone, wake,
//    wake_by_ref, and drop functions. The wake function sets the AtomicBool to
//    true.
//  - Arc::into_raw and Arc::from_raw are how you convert between Arc and the
//    raw *const () pointer the vtable functions receive. Key insight:
//    Arc::from_raw takes ownership - if you don't want it dropped at end of
//    scope, call Arc::into_raw again to release ownership back.
//  - The difference between wake and wake_by_ref: wake consumes the waker (let
//    the Arc drop), wake_by_ref borrows it (put it back with into_raw).
//  - The run loop: iterate tasks, skip ones not flagged as ready, poll ready
//    ones, remove completed ones. Loop until empty. Vec::retain_mut lets you
//    poll and remove in one pass.
//  - Initially flag all tasks as ready (they need at least one poll to start)
//  - TimerFuture stores an Arc<AtomicBool> for completion status and an
//    Arc<Mutex<Option<Waker>>> for the waker (Arc so the thread can access it).
//    poll stashes cx.waker().clone() in the mutex; the background thread reads
//    it back with .take() to call wake().
//  - TimerFuture needs a started flag so it only spawns one background thread.
//  - poll takes Pin<&mut Self>. If all your fields are Unpin, call
//    self.get_mut() once at the top of poll to unpin — calling it mid-method
//    consumes the Pin and blocks further access to self.
//  - cx.waker() is a method call, not field access.
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
//
// NOTE:
//
// Understanding Future, Poll, and Waker is the key to demystifying async Rust -
// and that's exactly what this assignment teaches. The gap between this and
// real Tokio is performance engineering, not conceptual.

use std::{
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread::{self, sleep},
    time::Duration,
};

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_fn, wake_fn, wake_by_ref_fn, drop_fn);

unsafe fn clone_fn(ptr: *const ()) -> RawWaker {
    unsafe {
        // Reconstruct the Arc from the raw pointer (takes ownership)
        let original = Arc::from_raw(ptr as *const AtomicBool);

        // Clone the Arc (increments the reference count)
        let cloned = original.clone();

        // Release ownership of the original back to the raw pointer
        // so it isn't dropped when this function returns
        let _ = Arc::into_raw(original);

        // Convert the clone into a raw pointer for the new RawWake
        let cloned_ptr = Arc::into_raw(cloned) as *const ();

        RawWaker::new(cloned_ptr, &VTABLE)
    }
}

unsafe fn wake_fn(ptr: *const ()) {
    unsafe {
        // Reconstruct the Arc from the raw pointer (takes ownership)
        let original = Arc::from_raw(ptr as *const AtomicBool);

        // Set the wake flag to indicate task is ready
        original.store(true, Relaxed);
    }
}

unsafe fn wake_by_ref_fn(ptr: *const ()) {
    unsafe {
        // Reconstruct the Arc from the raw pointer (takes ownership)
        let original = Arc::from_raw(ptr as *const AtomicBool);

        // Set the wake flag to indicate task is ready
        original.store(true, Relaxed);

        // Release ownership of the original back to the raw pointer
        // so it isn't dropped when this function returns
        let _ = Arc::into_raw(original);
    }
}

unsafe fn drop_fn(ptr: *const ()) {
    unsafe {
        // Reconstruct the Arc from the raw pointer and let it drop
        let _ = Arc::from_raw(ptr as *const AtomicBool);
    }
}

fn create_waker(wake_flag: Arc<AtomicBool>) -> Waker {
    let raw_ptr = Arc::into_raw(wake_flag) as *const ();
    unsafe { Waker::from_raw(RawWaker::new(raw_ptr, &VTABLE)) }
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    wake_flag: Arc<AtomicBool>,
}

struct MiniRuntime {
    tasks: Vec<Task>,
}

impl MiniRuntime {
    const SLEEP_DURATION_MS: u64 = 10;

    fn new() -> Self {
        MiniRuntime { tasks: Vec::new() }
    }

    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        // We have to Box and Pin the incoming future
        let future = Box::pin(future);
        // Initialize the wake flag with true so it's immediately polled
        let wake_flag = Arc::new(AtomicBool::new(true));
        // Add the task to the runtime
        let task = Task { future, wake_flag };
        self.tasks.push(task);
    }

    fn run(&mut self) {
        loop {
            // Loop as long as tasks exist
            if self.tasks.is_empty() {
                break;
            }

            self.tasks.retain_mut(|task| {
                // Check if task is ready
                if !task.wake_flag.load(Relaxed) {
                    return true;
                }

                // Poll the ready task
                task.wake_flag.store(false, Relaxed);

                let waker = create_waker(task.wake_flag.clone());
                let mut ctx = Context::from_waker(&waker);
                if let Poll::Ready(()) = task.future.as_mut().poll(&mut ctx) {
                    return false;
                }
                true
            });

            sleep(Duration::from_millis(MiniRuntime::SLEEP_DURATION_MS));
        }
    }
}

struct TimerFuture {
    duration: Duration,
    completed: Arc<AtomicBool>,
    waker: Arc<Mutex<Option<Waker>>>,
    started: bool,
}

impl TimerFuture {
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            completed: Arc::new(AtomicBool::new(false)),
            waker: Arc::new(Mutex::new(None)),
            started: false,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Unpin self to get a &mut Self reference. This is safe because all
        // fields (Arc, Mutex, bool, Duration) implement Unpin.
        let this = self.get_mut();

        // If the task completed, signal it's ready
        if this.completed.load(Relaxed) {
            return Poll::Ready(());
        }

        // Store the waker behind the mutex
        let mut guard = this.waker.lock().expect("Failed to acquire lock!");
        *guard = Some(cx.waker().clone());
        drop(guard);

        // Start the task in a new thread if it hasn't been started yet.
        if !this.started {
            this.started = true;

            let completed_clone = this.completed.clone();
            let waker_clone = this.waker.clone();
            let duration_clone = this.duration;

            thread::spawn(move || {
                thread::sleep(duration_clone);
                completed_clone.store(true, Relaxed);
                if let Some(waker) = waker_clone.lock().unwrap().take() {
                    waker.wake();
                }
            });
        }

        Poll::Pending
    }
}

fn main() {
    let mut rt = MiniRuntime::new();

    rt.spawn(async {
        println!("Task 1: starting");
        TimerFuture::new(Duration::from_secs(1)).await;
        println!("Task 1: done after 1s");
    });

    rt.spawn(async {
        println!("Task 2: starting");
        TimerFuture::new(Duration::from_millis(500)).await;
        println!("Task 2: done after 500ms");
    });

    rt.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn single_task_completes() {
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        let mut rt = MiniRuntime::new();
        rt.spawn(async move {
            TimerFuture::new(Duration::from_millis(50)).await;
            done_clone.store(true, Relaxed);
        });
        rt.run();

        assert!(done.load(Relaxed), "task should have completed");
    }

    #[test]
    fn shorter_task_finishes_first() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();

        let mut rt = MiniRuntime::new();
        rt.spawn(async move {
            TimerFuture::new(Duration::from_millis(200)).await;
            order1.lock().unwrap().push(1);
        });
        rt.spawn(async move {
            TimerFuture::new(Duration::from_millis(50)).await;
            order2.lock().unwrap().push(2);
        });
        rt.run();

        let result = order.lock().unwrap();
        assert_eq!(*result, vec![2, 1], "shorter task should finish first");
    }

    #[test]
    fn zero_duration_completes_immediately() {
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        let mut rt = MiniRuntime::new();
        rt.spawn(async move {
            TimerFuture::new(Duration::ZERO).await;
            done_clone.store(true, Relaxed);
        });
        rt.run();

        assert!(done.load(Relaxed), "zero-duration task should complete");
    }
}
