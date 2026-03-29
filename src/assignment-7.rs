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
//     - fn new(size: u32) -> Pool — creates the pool with size
//       pre-created connections
//       * Note: If the connections were *real* database connections, new would
//         need to be async.
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

use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinSet,
};

struct Connection {
    id: u32,
}

impl Connection {
    fn new(id: u32) -> Self {
        Self { id }
    }

    async fn query(&self, sql: &str) -> String {
        // Create a random delay to simulate work
        let delay = rand::random_range(50..=200);
        let duration = tokio::time::Duration::from_millis(delay);

        // Sleep for the duration
        let _ = tokio::time::sleep(duration).await;

        // Return a query result
        format!(
            r#"connection {}: query "{}" completed in {:?}"#,
            self.id, sql, duration
        )
    }
}

struct Pool {
    // Thread safe pool for taking and returning connections
    connections: Arc<Mutex<VecDeque<Connection>>>,
    // Semaphore tracks the number borrowed connection
    semaphore: Arc<Semaphore>,
}

impl Pool {
    fn new(size: u32) -> Self {
        // Create the connections
        let connections: VecDeque<Connection> =
            (0..size).map(|id| Connection::new(id)).collect();

        // Wrap the connections in a synchronization primitive
        let connections = Arc::new(Mutex::new(connections));

        // Use a semaphore to facilitate borrowing connections
        let semaphore = Arc::new(Semaphore::new(size as usize));

        Self {
            connections,
            semaphore,
        }
    }

    async fn get(&self) -> PoolGuard {
        // Create an owned permit to track in the PoolGuard
        let semaphore_clone = self.semaphore.clone();
        let permit = semaphore_clone
            .acquire_owned()
            .await
            .expect("could not acquire permit");

        let connection = self
            .connections
            .lock()
            .expect("poisoned lock in PoolGuard::get")
            .pop_front()
            .expect("connection queue empty!");

        let connections = Arc::clone(&self.connections);

        PoolGuard {
            connections: connections,
            connection: Some(connection),
            _permit: permit,
        }
    }
}

struct PoolGuard {
    connections: Arc<Mutex<VecDeque<Connection>>>,
    connection: Option<Connection>,
    _permit: OwnedSemaphorePermit,
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        let connection = self.connection.take();
        self.connections
            .lock()
            .expect("poisoned lock in drop")
            .push_back(connection.expect("missing connection"));
    }
}

impl Deref for PoolGuard {
    type Target = Connection;
    fn deref(&self) -> &<Self as Deref>::Target {
        self.connection.as_ref().expect("connection taken")
    }
}

// Simulate some database work
async fn simulate_database_query(task: u32, pool: Arc<Pool>) {
    let connection = pool.get().await;
    let query = format!("task {task} running a query");
    let message = connection.query(&query).await;
    println!("{message}");
}

#[tokio::main]
async fn main() {
    const POOL_SIZE: u32 = 3;
    const NUM_TASKS: u32 = 10;

    let connection_pool = Arc::new(Pool::new(POOL_SIZE));

    let mut tasks = JoinSet::new();
    for task in 0..NUM_TASKS {
        let pool_clone = Arc::clone(&connection_pool);
        tasks.spawn(simulate_database_query(task, pool_clone));
    }

    tasks.join_all().await;
}

#[cfg(test)]
mod pool_tests {
    use super::Pool;
    use std::{sync::Arc, time::Duration};
    use tokio::time::timeout;

    #[tokio::test]
    async fn returns_up_to_capacity_without_blocking() {
        const POOL_SIZE: u32 = 3;
        let pool = Arc::new(Pool::new(POOL_SIZE));

        // Check out POOL_SIZE connections
        let _c1 = pool.get().await;
        let _c2 = pool.get().await;
        let _c3 = pool.get().await;

        // Next checkout should block
        let result = timeout(Duration::from_millis(10), pool.get()).await;
        assert!(result.is_err(), "get() should have blocked");
    }

    #[tokio::test]
    async fn unblocks_when_connection_returned() {
        const POOL_SIZE: u32 = 1;
        let pool = Arc::new(Pool::new(POOL_SIZE));

        let c1 = pool.get().await;

        // Pool exhausted - should block
        let result = timeout(Duration::from_millis(10), pool.get()).await;
        assert!(result.is_err(), "get() should have blocked");

        // Drop one connection and verify another is returned without blocking
        drop(c1);

        // Should succeed now
        let result = timeout(Duration::from_millis(10), pool.get()).await;
        assert!(result.is_ok(), "get() should not have blocked");
    }

    #[tokio::test]
    async fn same_connection_reused() {
        const POOL_SIZE: u32 = 1;
        let pool = Arc::new(Pool::new(POOL_SIZE));

        let c1 = pool.get().await;
        let id1 = c1.id;
        drop(c1);

        let c2 = pool.get().await;
        let id2 = c2.id;
        drop(c2);

        assert_eq!(id1, id2, "same connection should be reused");
    }
}

#[cfg(test)]
mod connection_tests {
    use super::Connection;

    #[tokio::test(start_paused = true)]
    async fn test_query() {
        let connection = Connection::new(0);
        let msg = connection.query("NAME=test").await;
        let mut iter = msg.split_whitespace();

        assert_eq!(Some("connection"), iter.next());
        assert_eq!(Some("0:"), iter.next());
        assert_eq!(Some("query"), iter.next());
        assert_eq!(Some("\"NAME=test\""), iter.next());
        assert_eq!(Some("completed"), iter.next());
        assert_eq!(Some("in"), iter.next());
        assert!(iter.next().expect("bad test expectation").ends_with("ms"));
        assert_eq!(None, iter.next());
    }
}
