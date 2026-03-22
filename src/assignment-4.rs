// Assignment 4: Graceful Shutdown Orchestrator
//
// Objective: Extend your chat server (or build fresh) with proper graceful
// shutdown - stop accepting new connections, drain in-flight work, and exit
// cleanly.
//
// Requirements:
//
//  1. Start with your chat server from Assignment 3 (copy it to
//     assignment-4.rs)
//  2. When the user presses Ctrl+C, the server should:
//     - Stop accepting new connections (exit the accept loop)
//     - Broadcast a shutdown message to all connected clients: "Server is
//       shutting down..."
//     - Wait for all connected clients to disconnect (or for their tasks to
//       finish), but with a timeout of 5 seconds — if clients haven't
//       disconnected by then, force-close
//  3. Print "Server shut down gracefully." on exit
//  4. Use:
//     - tokio::signal::ctrl_c() to detect Ctrl+C
//     - tokio::select! in the accept loop to race between accepting and the
//       shutdown signal
//     - tokio::time::timeout to enforce the 5-second drain deadline
//     - A tokio_util::sync::CancellationToken or a broadcast channel to notify
//       client tasks they should wrap up
//
// Hints:
//
//  - The accept loop becomes: select! { conn = listener.accept() => { ... }, _
//    = signal::ctrl_c() => { break; } }
//  - You'll need a way to track active client tasks — collect JoinHandles, or
//    use a JoinSet
//  - tokio::time::timeout(Duration::from_secs(5), drain_all_tasks).await — if
//    it returns Err(_), the timeout elapsed
//  - CancellationToken from tokio-util is the cleanest pattern: create one,
//    clone it to each client task, and call .cancel() on shutdown. Each client
//    checks .cancelled() in their select!
//  - Add tokio-util to your Cargo.toml if using CancellationToken
//
// Grading criteria:
//
// - Ctrl+C triggers a clean shutdown, not an abrupt kill
// - Clients are notified of the shutdown
// - Server waits for clients (up to the timeout) before exiting
// - No panics, no orphaned tasks
// - Code clearly separates the shutdown logic from the normal operation
use message::Message;
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::broadcast::{Receiver, Sender, channel},
    task::JoinSet,
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use user_manager::UserManager;

const IP_ADDRESS: &str = "127.0.0.1";
const PORT: u16 = 8080;
const CAPACITY: usize = 100;
const TIMEOUT_DURATION_SEC: u64 = 5;

mod user_manager {
    pub struct UserManager {
        user_id: u32,
    }

    impl UserManager {
        pub fn new() -> Self {
            Self { user_id: 1 }
        }

        pub fn create(&mut self) -> String {
            let username = format!("User{}", self.user_id);
            self.user_id += 1;
            username
        }
    }
}

mod message {
    #[derive(Clone)]
    pub struct Message {
        pub sender: String,
        pub message: String,
    }

    impl Message {
        pub fn new(sender: String, message: String) -> Self {
            Self { sender, message }
        }
    }
}

fn handle_client_disconnect(sender: String, tx: &Sender<Message>) {
    let leave_message = Message::new(sender, "has left the chat".to_string());
    tx.send(leave_message)
        .map_err(|e| eprintln!("Failed to broadcast leave message: {}", e))
        .ok();
}

async fn handle_connection(
    stream: TcpStream,
    mut broadcast_rx: Receiver<Message>,
    c_token: CancellationToken,
    sender: String,
    broadcast_tx: Sender<Message>,
) {
    // Split the stream into reader and writer for concurrent handling
    let (rx, mut tx) = stream.into_split();
    let mut reader = BufReader::new(rx);

    // Broadcast a join message to all other clients
    let join_message =
        Message::new(sender.clone(), "has joined the chat.".to_string());

    if let Err(e) = broadcast_tx.send(join_message) {
        eprintln!("Failed to broadcast join message: {}", e);
        return;
    }

    loop {
        let mut buf = String::new();
        select! {
            // Read a line from the client
            Ok(n) = reader.read_line(&mut buf) => {
                if n == 0 {
                    // Client disconnected
                    handle_client_disconnect(sender.clone(), &broadcast_tx);
                    break;
                }
                // Broadcast the message to all other clients
                let message = Message::new(
                    sender.clone(), buf.trim().to_string()
                );
                if let Err(e) = broadcast_tx.send(message) {
                    eprintln!("Failed to broadcast message: {}", e);
                    break;
                }
            }
            Ok(msg) = broadcast_rx.recv() => {
                if msg.sender == sender {
                    // Don't send the message back to the sender
                    continue;
                }

                let msg = format!("{}: {}\r\n", msg.sender, msg.message);
                if let Err(e) = tx.write_all(msg.as_bytes()).await {
                    eprintln!("Failed to send message to {}: {}", sender, e);
                    break;
                };
            }
            _ = c_token.cancelled() => {
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let Ok(listener) = TcpListener::bind(format!("{IP_ADDRESS}:{PORT}")).await
    else {
        eprintln!("Failed to bind to {}:{}", IP_ADDRESS, PORT);
        return;
    };

    // Create a broadcast channel for message distribution
    let (broadcast_tx, _broadcast_rx): (Sender<Message>, Receiver<Message>) =
        channel(CAPACITY);

    // Track tasks
    let mut tasks = JoinSet::new();
    let c_token = CancellationToken::new();

    // Handles username generation
    let mut user_manager = UserManager::new();

    println!("Chat server running on {}:{}", IP_ADDRESS, PORT);
    loop {
        select! {
            conn = listener.accept() => {
                match conn {
                    Ok((stream, addr)) => {
                        println!("New client connected: {}", addr);

                        let sender = user_manager.create();
                        let broadcast_rx = broadcast_tx.subscribe();
                        let broadcast_tx = broadcast_tx.clone();
                        let c_token = c_token.clone();

                        // Spawn a new task to handle this client's connection
                        tasks.spawn(async move {
                            handle_connection(
                                stream, broadcast_rx, c_token, sender,
                                broadcast_tx
                            ).await
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                        continue;
                    }
                }
            },
            _ = signal::ctrl_c() => {
                let shutdown_message = Message::new(
                    "Server".to_string(),
                    "shutting down...".to_string()
                );
                if let Err(e) = broadcast_tx.send(shutdown_message) {
                    eprintln!("Failed to broadcast shutdown message: {}", e);
                    return;
                }
                c_token.cancel();
                break;
            }
        }
    }

    let _ = timeout(Duration::from_secs(TIMEOUT_DURATION_SEC), async {
        tasks.join_all().await
    })
    .await;

    println!("Server shut down gracefully.");
}
