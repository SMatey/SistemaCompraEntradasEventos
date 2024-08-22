use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);

    let message = "Hello from client!";
    writer.write_all(message.as_bytes()).await?;

    // Read the response from the server
    let mut buffer = String::new();
    buf_reader.read_to_string(&mut buffer).await?;

    println!("Received from server: {}", buffer);

    Ok(())
}
