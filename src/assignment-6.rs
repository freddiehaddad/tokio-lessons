// Assignment 6: Async Retry with Exponential Backoff
//
// Objective: Write a generic, reusable async retry utility and test it with
// #[tokio::test].
//
// Requirements:
//
//  1. Implement a retry function with this signature (or similar):
//     async fn retry<F, Fut, T, E>(
//         f: F,
//         max_retries: u32,
//     ) -> Result<T, E>
//     where
//         F: Fn() -> Fut,
//         Fut: Future<Output = Result<T, E>>,
//         E: std::fmt::Display,
//  2. Behavior:
//     - Call f() up to max_retries + 1 times (1 initial attempt + N retries)
//     - If f() returns Ok, return immediately
//     - If f() returns Err, wait before retrying using exponential backoff:
//       base_delay * 2^attempt (e.g., 100ms, 200ms, 400ms, 800ms...)
//     - Add jitter: randomize each delay by ±25% to avoid thundering herd
//     - Log each retry: "Retry {attempt}/{max_retries}: {error}, waiting
//       {delay}ms"
//     - If all retries are exhausted, return the last error
//  3. Write tests using #[tokio::test]:
//     - Test 1: A function that always succeeds -> returns Ok on first try, no
//       retries
//     - Test 2: A function that fails twice then succeeds -> returns Ok after 2
//       retries
//     - Test 3: A function that always fails -> returns Err after exhausting
//       all retries
//     - Test 4: Verify the number of attempts matches expectations (use an
//       AtomicU32 counter)
//     - Bonus Test 5: Verify that the total elapsed time is roughly consistent
//       with exponential backoff (not instant, not linear)
//
// Hints:
//
//  - The Fn() -> Fut bound means the closure can be called multiple times
//    (unlike FnOnce)
//  - For tests that need mutable state across calls (fail N times then
//    succeed), use Arc<AtomicU32> as a call counter
//  - tokio::time::sleep(Duration::from_millis(delay)) for the backoff wait
//  - Jitter: delay * rand::random_range(0.75..=1.25) or similar
//  - tokio::time::pause() can be used in tests to make time-based tests instant
//    (advanced, optional)
//  - Keep the retry function in a module — this is a utility you'd actually
//    reuse
//
// Grading criteria:
//
//  - Generic over any async Fn() -> Future<Output = Result<T, E>>
//  - Exponential backoff with jitter implemented correctly
//  - All 4+ tests pass
//  - Tests actually verify behavior (not just "it compiles")
//  - Clean separation between the retry utility and the tests
//
// Why this matters: Every production system that talks to external services
// needs retry logic. Writing it as a generic utility teaches you async
// closures, higher-rank trait bounds, and async testing — all at once.

#[tokio::main]
async fn main() {
    todo!("Solution not implemented");
}
