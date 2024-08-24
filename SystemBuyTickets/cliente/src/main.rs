use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::AsyncReadExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Establece la conexión con el servidor una vez
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    let mut buffer = [0; 1024];
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        // Limpiar la pantalla (solo en Windows, si usas otro sistema operativo, ajusta esto)
        std::process::Command::new("cmd").args(&["/c", "cls"]).status()?;

        // Leer la respuesta del servidor
        let bytes_read = stream.read(&mut buffer).await?;
        let responses = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", responses);

        // Leer entrada del usuario
        print!("-> ");
        io::stdout().flush().await?;
        let mut input = String::new();
        reader.read_line(&mut input).await?;

        let entrada = input.trim().to_string();

        // Verificar si el usuario ingresó '0' para salir
        if entrada == "0" {
            println!("Saliendo del sistema...");
            break;
        }

        // Enviar la entrada al servidor
        stream.write_all(entrada.as_bytes()).await?;
        stream.flush().await?;

        // Leer la respuesta del servidor (resultados de búsqueda)
        let bytes_read = stream.read(&mut buffer).await?;
        let result = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("{}", result);
    }

    Ok(())
}
