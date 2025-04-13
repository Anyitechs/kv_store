use command::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
mod command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on port 6379");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client: {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client {}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_client(socket: tokio::net::TcpStream) -> anyhow::Result<()> {
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
                // Handle GET
                writer.write_all(b"OK\n").await?;
            }
            Command::Set(key, value) => {
                // Handle SET
                writer.write_all(b"OK\n").await?;
            }
            Command::Delete(key) => {
                // Handle DELETE
                writer.write_all(b"OK\n").await?;
            }
            Command::Unknown => {
                writer.write_all(b"ERROR Unknown Command\n").await?;
            }
        }

        writer.write_all(line.as_bytes()).await?;
    }

    Ok(())
}
