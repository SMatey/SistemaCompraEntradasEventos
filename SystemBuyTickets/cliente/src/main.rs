use tokio::net::TcpStream;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, AsyncReadExt};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    let mut buffer = [0; 1024];
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        // Limpiar pantalla y mostrar menú
        std::process::Command::new("cmd").args(&["/c", "cls"]).status()?;

        // Leer respuesta del servidor
        let bytes_read = stream.read(&mut buffer).await?;
        let responses = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", responses);

        // Leer entrada del usuario
        print!("Elija una opción: ");
        io::stdout().flush().await?;
        let mut input = String::new();
        reader.read_line(&mut input).await?;
        let opcion = input.trim();

        // Enviar la opción al servidor
        stream.write_all(opcion.as_bytes()).await?;
        stream.flush().await?;

        // Leer la respuesta del servidor después de elegir categoría
        let bytes_read = stream.read(&mut buffer).await?;
        let responses = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", responses);

        if opcion == "0" {
            println!("Conexión terminada.");
            break;
        }

        // Selección de cantidad de asientos
        print!("Ingrese la cantidad de asientos (máx. 6): ");
        io::stdout().flush().await?;
        input.clear();
        reader.read_line(&mut input).await?;
        let cantidad = input.trim().parse::<u32>().unwrap_or(0);

        if cantidad == 0 || cantidad > 6 {
            println!("Cantidad no válida o fuera del rango permitido.");
            continue;
        }

        // Enviar la cantidad al servidor
        stream.write_all(input.trim().as_bytes()).await?;
        stream.flush().await?;

        // Leer la respuesta final del servidor
        let bytes_read = stream.read(&mut buffer).await?;
        let responses = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", responses);
    }

    Ok(())
}
