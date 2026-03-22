// Assignment 3: Chat Server with Channels
//
// Objective: Build a TCP chat server where multiple clients connect and
// messages are broadcast to all connected clients.
//
// Requirements:
//
//  1. Listen on 127.0.0.1:8080 using tokio::net::TcpListener
//  2. When a client connects, assign them a name (e.g., User1, User2, etc.)
//  3. Broadcast a join message to all other clients: "User3 has joined the
//     chat"
//  4. When a client sends a message, broadcast it to all other connected
//     clients (not back to the sender) in the format: "User3: hello everyone"
//  5. When a client disconnects, broadcast a leave message: "User3 has left the
//     chat"
//  6. Use tokio::sync::broadcast channel for message distribution
//  7. Handle errors gracefully — one client disconnecting should never crash
//     the server or affect other clients
//
// How to test it:
//
//  - Run your server with cargo run --bin hw3
//  - Open 2-3 terminals and connect with: telnet 127.0.0.1 8080 (or ncat, or
//    Test-NetConnection + a simple TCP client)
//  - Type messages in one terminal and see them appear in the others
//
// Hints:
//
//  - TcpListener::accept() in a loop to accept new connections
//  - tokio::spawn a new task per client
//  - tokio::io::AsyncBufReadExt gives you lines() on a BufReader for reading
//    line-by-line
//  - tokio::io::AsyncWriteExt gives you write_all() for sending
//  - TcpStream can be split into a reader and writer with
//    tcp_stream.into_split()
//  - broadcast::channel(capacity) — each receiver gets all messages sent after
//    it subscribes
//  - Use tokio::select! to simultaneously wait for: (a) a new line from the
//    client, and (b) a new broadcast message to send to the client
//
// Grading criteria:
//
//  - Multiple clients can connect and chat simultaneously
//  - Messages go to all other clients, not back to sender
//  - Join/leave notifications work
//  - One client disconnecting doesn't crash anything
//  - select! is used to handle reading and writing concurrently per client
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    select,
    sync::broadcast::{Receiver, Sender, channel},
};

const IP_ADDRESS: &str = "127.0.0.1";
const PORT: u16 = 8080;
const CAPACITY: usize = 100;

#[derive(Clone)]
struct Message {
    sender: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for message distribution
    let (broadcast_tx, _broadcast_rx): (Sender<Message>, Receiver<Message>) =
        channel(CAPACITY);
    let mut user_id = 1;

    let Ok(listener) = TcpListener::bind(format!("{IP_ADDRESS}:{PORT}")).await
    else {
        eprintln!("Failed to bind to {}:{}", IP_ADDRESS, PORT);
        return;
    };

    println!("Chat server running on {}:{}", IP_ADDRESS, PORT);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("New client connected: {}", addr);

                // Each client gets its own broadcast receiver
                let mut broadcast_rx = broadcast_tx.subscribe();
                // Clone the broadcast sender for this client task
                let broadcast_tx = broadcast_tx.clone();

                // Assign a username based on the user_id and increment it for
                // the next user
                let next_user_id = user_id;
                user_id += 1;
                let username = format!("User{}", next_user_id);

                // Spawn a new task to handle this client's connection
                tokio::spawn(async move {
                    // Split the stream into reader and writer for concurrent
                    // handling
                    let (rx, mut tx) = stream.into_split();
                    let mut reader = BufReader::new(rx);

                    // Broadcast a join message to all other clients
                    let join_message = Message {
                        sender: username.clone(),
                        content: "has joined the chat.".to_string(),
                    };
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
                                    let leave_message = Message {
                                        sender: username.clone(),
                                        content: "has left the chat".to_string(),
                                    };
                                    broadcast_tx
                                        .send(leave_message).map_err(
                                            |e| eprintln!("Failed to broadcast leave message: {}", e)
                                        ).ok();
                                    break;
                                }
                                // Broadcast the message to all other clients
                                let message = Message {
                                    sender: username.clone(),
                                    content: buf.trim().to_string(),
                                };
                                if let Err(e) = broadcast_tx.send(message) {
                                    eprintln!("Failed to broadcast message: {}", e);
                                    break;
                                }
                            }
                            Ok(msg) = broadcast_rx.recv() => {
                                if msg.sender == username {
                                    // Don't send the message back to the sender
                                    continue;
                                }

                                let msg = format!("{}: {}\r\n", msg.sender, msg.content);
                                if let Err(e) = tx.write_all(msg.as_bytes()).await {
                                    eprintln!("Failed to send message to {}: {}", username, e);
                                    break;
                                };

                            }
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
