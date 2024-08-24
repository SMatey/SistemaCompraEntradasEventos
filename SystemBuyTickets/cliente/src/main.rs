use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use std::error::Error;
use tokio::io::stdin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        // Establece la conexión con el servidor
        let mut stream = TcpStream::connect("127.0.0.1:7878").await?;

        let mut buffer = [0; 1024];
        let stdin = stdin();
        let mut reader = BufReader::new(stdin);

        // Limpiar la pantalla (solo en Windows, si usas otro sistema operativo, ajusta esto)
        std::process::Command::new("cmd").args(&["/c", "cls"]).status()?;

        // Leer la respuesta del servidor
        let bytes_read = stream.read(&mut buffer).await?;
        let responses = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", responses);

        // Leer entrada del usuario
        print!("-> ");
        let mut input = String::new();
        reader.read_line(&mut input).await?;

        let entrada = input.trim().to_string(); // Asegúrate de que la entrada sea un String

        // Verificar si el usuario ingresó '0' para salir
        if entrada == "0" {
            println!("Saliendo del sistema...");
            break;
        }

        // Enviar la entrada al servidor
        stream.write_all(entrada.as_bytes()).await?;
        stream.flush().await?;
    }

    Ok(())
}
