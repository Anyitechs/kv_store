use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use command::Command;
use store::KeyValueStore;
use std::sync::Arc;

mod command;
mod store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on port 6379");

    let store = Arc::new(KeyValueStore::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client: {:?}", addr);
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store).await {
                eprintln!("Error handling client {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_client(socket: tokio::net::TcpStream, store: Arc<KeyValueStore>) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line: String = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            // Connection closed
            break;
        }

        println!("Received: {}", line.trim());
        let command = Command::parse(&line);

        match command {
            Command::Get(key) => {
                if let Some(value) = store.get(&key).await {
                    writer.write_all(format!("VALUE {}\n", value).as_bytes()).await?;
                } else {
                    writer.write_all(b"ERROR key not found\n").await?;
                }
            }
            Command::Set(key, value) => {
                store.set(key, value).await;
                writer.write_all(b"OK\n").await?;
            }
            Command::Delete(key) => {
                if store.delete(&key).await {
                    writer.write_all(b"OK\n").await?;
                } else {
                    writer.write_all(b"ERROR Key not found\n").await?;
                }
            }
            Command::Unknown => {
                writer.write_all(b"ERROR Unknown Command\n").await?;
            }
        }
    }

    Ok(())
}
