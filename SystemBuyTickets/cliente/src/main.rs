use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);

    // Leer y mostrar el menú del servidor línea por línea
    let mut buffer = String::new();
    while buf_reader.read_line(&mut buffer).await? > 0 {
        print!("{}", buffer);
        buffer.clear();  // Limpiar buffer para la siguiente línea
    }

    // Ahora enviar una opción al servidor
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    writer.write_all(input.as_bytes()).await?;

    // Leer y mostrar la respuesta del servidor
    let mut response = String::new();
    while buf_reader.read_line(&mut response).await? > 0 {
        print!("{}", response);
        response.clear();  // Limpiar buffer para la siguiente línea
    }

    Ok(())
}


