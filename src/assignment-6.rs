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
//    - tokio::time::pause() requires the "test-util" feature flag.
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

use retry::retry;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, atomic::AtomicU32};

mod retry {
    use rand::random_range;
    use tokio::time::Duration;
    use tokio::time::sleep;

    fn calculate_backoff(attempt: u32) -> f64 {
        const BASE_DELAY_MS: f64 = 100.0;
        BASE_DELAY_MS * 2_f64.powi(attempt as i32) * random_range(0.75..=1.25)
    }

    pub async fn retry<F, Fut, T, E>(f: F, max_retries: u32) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    if attempt == max_retries {
                        return Err(e);
                    }

                    let backoff_delay_ms = calculate_backoff(attempt);
                    let backoff_duration_ms =
                        Duration::from_millis(backoff_delay_ms as u64);

                    attempt += 1;
                    eprintln!(
                        "Retry {}/{}: {}, waiting {:?}",
                        attempt, max_retries, e, backoff_duration_ms
                    );

                    sleep(backoff_duration_ms).await;
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::retry::retry;

        use std::sync::{
            Arc,
            atomic::{AtomicU32, Ordering::Relaxed},
        };
        use tokio::time::Instant;

        async fn test_function(
            c: &AtomicU32,
            expected: u32,
        ) -> Result<u32, u32> {
            let count = c.load(Relaxed);
            if count == expected {
                Ok(count)
            } else {
                c.fetch_add(1, Relaxed);
                Err(count)
            }
        }

        #[tokio::test]
        async fn succeed_first_try() {
            let attempts = Arc::new(AtomicU32::new(0));
            let expected = 0;
            let max_retries = 0;

            let result =
                retry(|| test_function(&attempts, 0), max_retries).await;
            assert_eq!(Ok(expected), result);
        }

        #[tokio::test]
        async fn succeed_on_third_try() {
            let attempts = Arc::new(AtomicU32::new(0));
            let expected = 2;
            let max_retries = 2;

            let result =
                retry(|| test_function(&attempts, expected), max_retries).await;
            assert_eq!(Ok(expected), result);
        }

        #[tokio::test]
        async fn fail_after_max_retries() {
            let attempts = Arc::new(AtomicU32::new(0));
            let expected = 2;
            let max_retries = 1;

            let result =
                retry(|| test_function(&attempts, expected), max_retries).await;
            assert_eq!(Err(1), result);
        }

        #[tokio::test]
        async fn check_expected_attempts() {
            let attempts = Arc::new(AtomicU32::new(0));
            let expected = 3;
            let max_retries = 5;

            let result =
                retry(|| test_function(&attempts, expected), max_retries).await;
            assert_eq!(Ok(expected), result);
            assert_eq!(attempts.load(Relaxed), expected);
        }

        #[tokio::test(start_paused = true)]
        async fn verify_exponential_backoff_consistency() {
            let lower_bound_ms: f64 =
                (0..5).map(|i| 100.0 * 2_f64.powi(i) * 0.75).sum();
            let upper_bound_ms: f64 =
                (0..5).map(|i| 100.0 * 2_f64.powi(i) * 1.25).sum();
            let attempts = Arc::new(AtomicU32::new(0));
            let expected = 5;
            let max_retries = 5;

            let start_time = Instant::now();

            let result =
                retry(|| test_function(&attempts, expected), max_retries).await;
            assert_eq!(Ok(expected), result);

            let elapsed_ms = start_time.elapsed().as_millis() as f64;
            assert!(
                elapsed_ms >= lower_bound_ms,
                "Too fast: {elapsed_ms}ms < {lower_bound_ms}ms"
            );
            assert!(
                elapsed_ms <= upper_bound_ms,
                "Too slow: {elapsed_ms}ms > {upper_bound_ms}ms"
            );
        }
    }
}

async fn example(c: &AtomicU32, e: u32) -> Result<String, String> {
    let current = c.load(Relaxed);
    if current == e {
        Ok(format!("Count is {}!", current))
    } else {
        c.fetch_add(1, Relaxed);
        Err(format!("Count is {}!", current))
    }
}

#[tokio::main]
async fn main() {
    let count = Arc::new(AtomicU32::new(0));
    let result = retry(|| example(&count, 5), 10).await;
    println!("Example successful on third attempt -> {:?}", result);
}
