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

| #   | Name                           | Key Concepts                                             |
| --- | ------------------------------ | -------------------------------------------------------- |
| 1   | Concurrent Web Fetcher         | `tokio::spawn`, `JoinHandle`, basic async error handling |
| 2   | Rate-Limited Task Queue        | `Semaphore`, bounded concurrency, backpressure           |
| 3   | Chat Server with Channels      | `TcpListener`, `broadcast` channel, `select!`            |
| 4   | Graceful Shutdown Orchestrator | `CancellationToken`, `signal::ctrl_c`, `timeout`         |
| 5   | Producer-Consumer Pipeline     | Bounded `mpsc` channels, multi-stage pipelines           |
| 6   | Async Retry with Backoff       | Async generics, `tokio::time`, `#[tokio::test]`          |

> NOTE: Assignments with an asterisk (*) indicate no solution _yet_.

## Getting Started

Each assignment can be run with:

```console
cargo run --bin hw1
```

Replace `hw1` with the assignment number (e.g., `hw2`, `hw3`, etc.).

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) — official walkthrough from
  setup to a working mini-Redis
- [Tokio API Docs](https://docs.rs/tokio/latest/tokio/) — reference for all
  modules and types
- [mini-redis](https://github.com/tokio-rs/mini-redis) — a real-world example
  project using many of these patterns
