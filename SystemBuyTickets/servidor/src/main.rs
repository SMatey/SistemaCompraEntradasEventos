use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

// Estructuras necesarias
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoSilla {
    Disponible,
    Reservada,
    Comprada,
}

#[derive(Debug, Clone)]
pub struct Silla {
    pub numero: u32,
    pub estado: EstadoSilla,
}

#[derive(Debug, Clone)]
pub struct Zona {
    pub nombre: String,
    pub filas: HashMap<u32, Vec<Silla>>,
}

#[derive(Debug, Clone)]
pub struct Categoria {
    pub nombre: String,
    pub zonas: Vec<Zona>,
}

#[derive(Debug, Clone)]
pub struct Estadio {
    pub categorias: Vec<Categoria>,
}

// Inicializar el mapeo del estadio
async fn inicializar_mapeo() -> Estadio {
    let mut categorias = Vec::new();

    let nombres_categorias = vec!["Platea Este", "Platea Oeste", "General Norte", "General Sur"];
    
    for nombre_categoria in nombres_categorias {
        let mut categoria = Categoria {
            nombre: nombre_categoria.to_string(),
            zonas: Vec::new(),
        };

        // Crear zonas A, B y C para cada categoría
        for nombre_zona in vec!["Zona A", "Zona B", "Zona C"] {
            let mut zona = Zona {
                nombre: nombre_zona.to_string(),
                filas: HashMap::new(),
            };

            // Agregar filas y sillas a cada zona
            let fila_1: Vec<Silla> = (1..=20).map(|num| Silla {
                numero: num,
                estado: match num % 5 {
                    0 => EstadoSilla::Reservada,
                    1 => EstadoSilla::Comprada,
                    _ => EstadoSilla::Disponible,
                },
            }).collect();
            
            let fila_2: Vec<Silla> = (21..=40).map(|num| Silla {
                numero: num,
                estado: match num % 6 {
                    0 => EstadoSilla::Comprada,
                    1 => EstadoSilla::Reservada,
                    _ => EstadoSilla::Disponible,
                },
            }).collect();
            
            zona.filas.insert(1, fila_1);
            zona.filas.insert(2, fila_2);

            categoria.zonas.push(zona);
        }

        categorias.push(categoria);
    }

    Estadio { categorias }
}


// Estructura para datos del cliente
#[derive(Debug)]
struct Solicitud {
    indice_categoria: usize,
    cantidad_boletos: u32,
    confirmar_compra: bool,
}

// Función para deserializar y validar la solicitud
fn deserializar_solicitud(datos: &str) -> Result<Solicitud, String> {
    let partes: Vec<&str> = datos.trim().split(',').collect();

    if partes.len() != 3 {
        return Err("Formato de solicitud incorrecto".to_string());
    }

    let indice_categoria = partes[0].parse::<usize>().map_err(|_| "Índice de categoría inválido".to_string())?;
    let cantidad_boletos = partes[1].parse::<u32>().map_err(|_| "Cantidad de boletos inválida".to_string())?;

    let confirmar_compra = match partes[2].trim().to_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return Err("Valor de confirmación inválido".to_string()),
    };

    Ok(Solicitud {
        indice_categoria,
        cantidad_boletos,
        confirmar_compra,
    })
}

// Función para manejar al cliente
async fn manejar_cliente(mut stream: TcpStream, estadio: Arc<Mutex<Estadio>>) {
    let mut buffer = [0; 1024];
    let bytes_leidos = match stream.read(&mut buffer).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error al leer del stream: {:?}", e);
            return;
        }
    };

    let datos = String::from_utf8_lossy(&buffer[..bytes_leidos]);
    let solicitud = match deserializar_solicitud(&datos) {
        Ok(s) => s,
        Err(e) => {
            let respuesta = format!("Error al deserializar solicitud: {}", e);
            if let Err(e) = stream.write_all(respuesta.as_bytes()).await {
                eprintln!("Error al escribir al stream: {:?}", e);
            }
            return;
        }
    };

    let Solicitud {
        indice_categoria,
        cantidad_boletos,
        confirmar_compra,
    } = solicitud;

    // Bloquear el estadio para modificarlo
    let mut estadio = estadio.lock().await;

    // Buscar los mejores asientos disponibles
    let (mut asientos_recomendados, mensaje) = estadio.buscar_asientos(indice_categoria, cantidad_boletos, 10);

    // Crear mensaje de respuesta para el cliente
    let mut respuesta = format!("{}\n", mensaje);
    if !asientos_recomendados.is_empty() {
        respuesta.push_str("Asientos recomendados:\n");
        for silla in &asientos_recomendados {
            respuesta.push_str(&format!("Número: {}, Estado: {:?}\n", silla.numero, silla.estado));
        }

        // Reservar los asientos recomendados
        estadio.reservar_sillas(&mut asientos_recomendados);

        // Confirmar o cancelar la compra
        if confirmar_compra {
            estadio.confirmar_compra_sillas(indice_categoria, &mut asientos_recomendados, true);
            respuesta.push_str("\nCompra realizada.");
        } else {
            estadio.confirmar_compra_sillas(indice_categoria, &mut asientos_recomendados, false);
            respuesta.push_str("\nReserva cancelada, asientos puestos de nuevo disponibles.");
        }
    } else {
        respuesta.push_str("No se encontraron suficientes asientos disponibles.");
    }

    // Enviar la respuesta al cliente
    if let Err(e) = stream.write_all(respuesta.as_bytes()).await {
        eprintln!("Error al escribir al stream: {:?}", e);
    }
}


//-------------MAIN PRINCIPAL
#[tokio::main]
async fn main() {
    let estadio = Arc::new(Mutex::new(inicializar_mapeo().await));
    let listener = TcpListener::bind(format!("127.0.0.1:7878"))
        .await
        .expect("Error al bindear el puerto");

    println!("Servidor escuchando en el puerto 7878");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let estadio = estadio.clone();
                tokio::spawn(async move {
                    manejar_cliente(stream, estadio).await;
                });
            }
            Err(e) => eprintln!("Error al aceptar conexión: {:?}", e),
        }
    }
}


//=======================FUNCIONES DE BUSQUEDA, MODIFICACION DE ESTADO========================
impl Estadio {
    // Función para buscar los mejores asientos disponibles en una categoría
    fn buscar_asientos(&mut self, indice_categoria: usize, cantidad_boletos: u32, max_boletos: u32) -> (Vec<Silla>, String) {
        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string());
        }

        let categoria = &mut self.categorias[indice_categoria];
        let mut asientos_recomendados = Vec::new();
        let mut mensaje = String::from("");

        // Itera sobre las zonas y filas de la categoría seleccionada
        for zona in &mut categoria.zonas {
            for (numero_fila, asientos) in &mut zona.filas {
                let mut asientos_disponibles: Vec<&mut Silla> = asientos.iter_mut()
                    .filter(|silla| silla.estado == EstadoSilla::Disponible)
                    .collect();

                // Si hay suficientes asientos disponibles, los selecciona
                if asientos_disponibles.len() >= cantidad_boletos as usize {
                    for silla in asientos_disponibles.iter_mut().take(cantidad_boletos as usize) {
                        asientos_recomendados.push((*silla).clone());
                    }
                    mensaje = format!("Asientos encontrados en la zona '{}', fila '{}'", zona.nombre, numero_fila);
                    self.reservar_sillas(&mut asientos_recomendados);
                    return (asientos_recomendados, mensaje);
                }
            }
        }

        mensaje = "No se encontraron suficientes asientos disponibles".to_string();
        (Vec::new(), mensaje)
    }

    // Función para reservar asientos (cambia su estado a Reservada)
    fn reservar_sillas(&mut self, asientos: &mut Vec<Silla>) {
        for silla in asientos.iter_mut() {
            silla.estado = EstadoSilla::Reservada;
        }
    }

    // Función para confirmar o cancelar la compra de asientos
    fn confirmar_compra_sillas(&mut self, indice_categoria: usize, asientos: &mut Vec<Silla>, confirmar: bool) {
        if indice_categoria >= self.categorias.len() {
            eprintln!("Categoría no válida");
            return;
        }

        let categoria = &mut self.categorias[indice_categoria];

        for zona in &mut categoria.zonas {
            for (_, fila_asientos) in &mut zona.filas {
                for silla in fila_asientos.iter_mut() {
                    if asientos.iter().any(|a| a.numero == silla.numero) {
                        silla.estado = if confirmar {
                            EstadoSilla::Comprada
                        } else {
                            EstadoSilla::Disponible
                        };
                    }
                }
            }
        }
    }
}
