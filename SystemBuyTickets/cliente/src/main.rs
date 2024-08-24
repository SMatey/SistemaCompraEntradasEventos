use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Definir las solicitudes que se van a enviar
    let solicitudes = vec![
        (0, 2, "true"),   // Solicitud 1: Categoría 0, 2 boletos, confirmar compra
        (1, 4, "false"),  // Solicitud 2: Categoría 1, 4 boletos, cancelar
        (0, 1, "true"),   // Solicitud 3: Categoría 0, 1 boleto, confirmar compra
    ];

    // Iterar sobre cada solicitud
    for (indice_categoria, cantidad_boletos, confirmar_compra) in solicitudes {
        // Conectar al servidor
        let mut stream = TcpStream::connect("127.0.0.1:7878").await?;

        // Enviar datos al servidor
        let mensaje = format!("{},{},{}", indice_categoria, cantidad_boletos, confirmar_compra);
        stream.write_all(mensaje.as_bytes()).await?;

        // Leer respuesta del servidor
        let mut buffer = [0; 1024];
        let bytes_leidos = stream.read(&mut buffer).await?;
        let respuesta = String::from_utf8_lossy(&buffer[..bytes_leidos]);

        // Imprimir la respuesta
        println!("Respuesta del servidor: {}", respuesta);
    }

    Ok(())
}
