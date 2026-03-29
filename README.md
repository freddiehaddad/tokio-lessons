# Learn Tokio

A collection of progressive assignments designed to teach asynchronous
programming in Rust using [Tokio](https://tokio.rs/). Each assignment builds on
concepts from previous ones, gradually introducing new async primitives and
patterns.

Each assignment file contains the full requirements and hints at the top. Try to
implement each one yourself before looking at the solution code in `src/`.

## Prerequisites

- Comfortable with Rust fundamentals (ownership, borrowing, traits, generics,
  error handling)
- Basic understanding of async/await syntax in Rust

## Assignments

| #   | Name                           | Key Concepts                                             | Solution |
| --- | ------------------------------ | -------------------------------------------------------- | -------- |
| [1] | Concurrent Web Fetcher         | `tokio::spawn`, `JoinHandle`, basic async error handling | ✓        |
| [2] | Rate-Limited Task Queue        | `Semaphore`, bounded concurrency, backpressure           | ✓        |
| [3] | Chat Server with Channels      | `TcpListener`, `broadcast` channel, `select!`            | ✓        |
| [4] | Graceful Shutdown Orchestrator | `CancellationToken`, `signal::ctrl_c`, `timeout`         | ✓        |
| [5] | Producer-Consumer Pipeline     | Bounded `mpsc` channels, multi-stage pipelines           | ✓        |
| [6] | Async Retry with Backoff       | Async generics, `tokio::time`, `#[tokio::test]`          | ✓        |
| [7] | Connection Pool                | `Mutex`, `Semaphore`, `Deref`/`Drop`, guard pattern      | ✓        |
| [8]*| Custom Mini-Runtime (Bonus)    | `Future`, `Waker`, `RawWaker`, `Poll`, `Pin`             | ○        |

## Getting Started

Each assignment can be run with:

```console
cargo run --bin hw1
```

Tests can be run with:

```console
cargo test --bin hw6
```

> **NOTE:** Replace `hw1` with the assignment number (e.g., `hw2`, `hw3`, etc.).

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) — official walkthrough from
  setup to a working mini-Redis
- [Tokio API Docs](https://docs.rs/tokio/latest/tokio/) — reference for all
  modules and types
- [mini-redis](https://github.com/tokio-rs/mini-redis) — a real-world example
  project using many of these patterns

[1]: ./src/assignment-1.rs
[2]: ./src/assignment-2.rs
[3]: ./src/assignment-3.rs
[4]: ./src/assignment-4.rs
[5]: ./src/assignment-5.rs
[6]: ./src/assignment-6.rs
[7]: ./src/assignment-7.rs
[8]: ./src/assignment-8.rs
