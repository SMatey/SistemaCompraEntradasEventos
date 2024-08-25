use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{Duration, Instant};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Definir las solicitudes que se van a enviar
    let solicitudes = vec![
        (0, 10, "true"),   // Solicitud 1: Categoría 0, 10 boletos, confirmar compra
        (0, 5, "false"),   // Solicitud 2: Categoría 0, 5 boletos, cancelar
        (1, 10, "true"),   // Solicitud 3: Categoría 1, 10 boletos, confirmar compra
        (1, 10, "false"),  // Solicitud 4: Categoría 1, 10 boletos, cancelar
        (2, 7, "true"),    // Solicitud 5: Categoría 2, 7 boletos, confirmar compra
        (2, 3, "true"),    // Solicitud 6: Categoría 2, 3 boletos, confirmar compra
        (3, 10, "false"),  // Solicitud 7: Categoría 3, 10 boletos, cancelar
        (3, 11, "true"),   // Solicitud 8: Categoría 3, 10 boletos, confirmar compra
    ];

    // Crear una lista de tareas para enviar solicitudes concurrentemente
    let mut handles = Vec::new();
    let start = Instant::now();

    for (indice_categoria, cantidad_boletos, confirmar_compra) in solicitudes {
        let handle = tokio::spawn(async move {
            let mut stream = match TcpStream::connect("127.0.0.1:7878").await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Error al conectar: {:?}", e);
                    return;
                }
            };

            let mensaje = format!("{},{},{}", indice_categoria, cantidad_boletos, confirmar_compra);
            if let Err(e) = stream.write_all(mensaje.as_bytes()).await {
                eprintln!("Error al enviar mensaje: {:?}", e);
                return;
            }

            let mut buffer = [0; 1024];
            let bytes_leidos = match stream.read(&mut buffer).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error al leer respuesta: {:?}", e);
                    return;
                }
            };

            let respuesta = String::from_utf8_lossy(&buffer[..bytes_leidos]);
            println!("Respuesta del servidor: {}", respuesta);
        });

        handles.push(handle);
    }

    // Esperar a que todas las tareas se completen
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    println!("Tiempo total de ejecución: {:?}", duration);

    Ok(())
}