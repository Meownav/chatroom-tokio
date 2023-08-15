use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    println!("Server now listening...");

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    let (tx, _rx) = broadcast::channel(10);
    let client_count = Arc::new(AtomicUsize::new(0)); // Counter for connected clients

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        client_count.fetch_add(1, Ordering::Relaxed); // Increment the counter

        let client_count_clone = client_count.clone();

        tokio::spawn(async move {
            let mut rx = tx.subscribe();
            let join_msg = format!("[{}] has joined the room.", addr);
            tx.send((join_msg.clone(), addr)).unwrap();

            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) =>{
                        if result.unwrap() == 0 {
                            client_count_clone.fetch_sub(1, Ordering::Relaxed);  // Decrement the counter
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() =>{
                        let (msg, other_addr) = result.unwrap();
                        // Check the client count to determine whether to show the message
                        if addr != other_addr || client_count_clone.load(Ordering::Relaxed) == 1 {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
