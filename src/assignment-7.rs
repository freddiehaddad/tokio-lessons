// Assignment 7: Connection Pool
//
// Objective: Implement a simple async connection pool that manages a fixed set
// of reusable connections, with callers waiting when none are available.
//
// Scenario: You're building a database connection pool. Connections are
// expensive to create, so you want to reuse them. Multiple tasks need
// connections concurrently, but the pool has a fixed size.
//
// Requirements:
//
//  1. Define a Connection struct that simulates a database connection:
//     - Has an id: u32 field
//     - Has an async fn query(&self, sql: &str) -> String method that simulates
//       work (sleep 50-200ms, return a result string)
//  2. Implement a Pool struct with:
//     - async fn new(size: u32) -> Pool — creates the pool with size
//       pre-created connections
//     - async fn get(&self) -> PoolGuard — checks out a connection. If none are
//       available, waits until one is returned
//     - When the PoolGuard is dropped, the connection is automatically returned
//       to the pool
//  3. PoolGuard behavior:
//     - Wraps a connection and a reference back to the pool
//     - Implements Deref so you can call guard.query(...) directly
//     - On drop, returns the connection to the pool
//  4. Use these Tokio primitives:
//     - tokio::sync::Mutex to protect the pool's internal connection list
//     - tokio::sync::Semaphore to limit concurrent checkouts and wake waiters
//       (or Notify — your choice)
//  5. Write a main that demonstrates:
//     - Create a pool of size 3
//     - Spawn 10 tasks that each check out a connection, run a query, and return it
//     - Print which connection each task got and when
//     - Show that at most 3 tasks hold connections at any time
//  6. Write tests:
//     - Pool returns connections up to its capacity without blocking
//     - A task blocks when the pool is exhausted, and proceeds once a
//       connection is returned
//     - Connections are reused (same IDs appear multiple times)
//
// Hints:
//
//  - Semaphore is ideal here: initialize with size permits. get() acquires a
//    permit, pops a connection. PoolGuard::drop pushes the connection back and
//    releases the permit.
//  - For returning the connection on drop, PoolGuard needs an
//    Option<Connection> (so you can .take() it in Drop) and an Arc reference to
//    the pool internals.
//  - Drop is synchronous — you can't .await inside it. Use a non-async Mutex
//    (like std::sync::Mutex or parking_lot::Mutex) for the connection list, or
//    use tokio::spawn to return the connection asynchronously.
//  - Deref<Target = Connection> makes the guard ergonomic to use.
//
// Grading criteria:
//
//  - Pool correctly limits concurrent access to N connections
//  - Callers block (not error) when pool is exhausted
//  - Connections are returned and reused
//  - Guard pattern with Deref and Drop is implemented
//  - Tests verify the above behaviors
//  - No deadlocks, no panics
//
// Why this matters: Connection pools are everywhere — databases, HTTP clients,
// gRPC channels. Understanding the async primitives behind them (semaphore +
// mutex + guard pattern) is fundamental to building production Rust services.
//
// This is the toughest assignment yet — the Drop + async interaction is tricky.
// Take your time!

#[tokio::main]
async fn main() {
    todo!("Solution not implemented yet");
}
