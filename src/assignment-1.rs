// Assignment 1: Concurrent Web Fetcher
//
// Objective: Build a CLI tool that fetches multiple URLs concurrently and
// reports results.
//
// Requirements:
//
//  1. Accept a hardcoded list of at least 5 URLs (or take them from
//     command-line args - your choice)
//  2. Fetch all URLs concurrently using tokio::spawn and reqwest
//  3. For each URL, print:
//      - The URL
//      - The HTTP status code (or the error if the request failed)
//      - How long that individual request took
//  4. After all requests complete, print the total elapsed time
//  5. Handle errors gracefully — a single failed URL should not crash the
//     program
//
// Hints:
//
//  - You'll need to add tokio (with full features) and reqwest to your
//    Cargo.toml
//  - std::time::Instant is fine for timing
//  - Think about what type JoinHandle returns and how to collect results
//
// Grading criteria:
//
//  - All URLs fetched concurrently (not sequentially!)
//  - Errors are handled, not unwrap()'d
//  - Clean, idiomatic code
use anyhow::Result;
use reqwest::StatusCode;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug)]
struct MyResponse {
    url: String,
    status: StatusCode,
    time_taken: Duration,
}

// Function to fetch a URL and return a MyResponse struct
async fn fetch_url(url: String) -> Result<MyResponse> {
    let start_time = Instant::now();
    let resp = reqwest::get(&url).await?;
    let status = resp.status();
    let time_taken = start_time.elapsed();
    Ok(MyResponse {
        url,
        status,
        time_taken,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();

    // The URLS to fetch
    let urls = vec![
        "https://httpbin.org",
        "https://jsonplaceholder.typicode.com",
        "https://api.ipify.org/",
        "https://quotes.toscrape.com/",
        "https://api.weather.gov/",
        "https://a.bad.url",
    ];

    // Create a channel to receive responses
    let (tx, mut rx) = mpsc::channel(urls.len());

    // Spawn a task for each URL
    for url in urls {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let result = fetch_url(url.to_string()).await;
            let _ = tx_clone
                .send(result.map_err(|e| anyhow::anyhow!("{url}: {e}")))
                .await;
        });
    }

    // Drop the original sender to close the channel when all tasks are done
    drop(tx);

    // Collect and print responses as they arrive
    while let Some(result) = rx.recv().await {
        match result {
            Ok(response) => println!(
                "url: {}, status: {}, response time: {}",
                response.url,
                response.status,
                response.time_taken.as_millis()
            ),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Print total elapsed time
    let end_time = start_time.elapsed();
    println!("Total elapsed time: {}ms", end_time.as_millis());

    Ok(())
}
