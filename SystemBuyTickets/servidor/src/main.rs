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
            let fila_1: Vec<Silla> = (1..=10).map(|num| Silla {
                numero: num,
                estado: match num % 5 {
                    0 => EstadoSilla::Reservada,
                    1 => EstadoSilla::Comprada,
                    _ => EstadoSilla::Disponible,
                },
            }).collect();
            
            let fila_2: Vec<Silla> = (11..=20).map(|num| Silla {
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

    // Buscar los mejores asientos disponibles, limite de 10 tickets
    let (mut asientos_recomendados, mensaje, nombre_categoria) = estadio.buscar_asientos(indice_categoria, cantidad_boletos, 10);

    // Crear mensaje de respuesta para el cliente
    let mut respuesta = format!("Categoría elegida: {}\n", nombre_categoria);
    respuesta.push_str(&format!("{}\n", mensaje));

    if !asientos_recomendados.is_empty() {
        respuesta.push_str("Asientos recomendados:\n");
        for (zona_nombre, numero_fila, silla) in &asientos_recomendados {
            respuesta.push_str(&format!("Zona: {}, Fila: {}, Asiento: {}\n", zona_nombre, numero_fila, silla.numero));
        }

        // Confirmar o cancelar la compra
        if confirmar_compra {
            // Cambiar el estado de las sillas a Comprada
            estadio.confirmar_compra_sillas(indice_categoria, &mut asientos_recomendados, true);
            respuesta.push_str("\nCompra realizada.");
        } else {
            // Cambiar el estado de las sillas a Disponible (cancelación)
            estadio.confirmar_compra_sillas(indice_categoria, &mut asientos_recomendados, false);
            respuesta.push_str("\nReserva cancelada, asientos puestos de nuevo disponibles.");
        }
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
                    //Mensaje que se muestra cada que se recibe una solicitud
                    println!("-------------Solicitud Recibida.\n");
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
    fn buscar_asientos(&mut self, indice_categoria: usize, cantidad_boletos: u32, max_boletos: u32) -> (Vec<(String, u32, Silla)>, String, String) {
        // Verificación de la cantidad de boletos permitidos
        if cantidad_boletos > max_boletos {
            return (Vec::new(), "Transacción no permitida: Excede el máximo de asientos permitidos para comprar".to_string(), String::new());
        }

        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string(), String::new());
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

                // Añadir asientos hasta completar la cantidad solicitada
                for silla in asientos_disponibles.iter_mut().take(cantidad_boletos as usize - asientos_recomendados.len()) {
                    asientos_recomendados.push((zona.nombre.clone(), *numero_fila, silla.clone()));
                    silla.estado = EstadoSilla::Reservada; // Actualiza el estado de la silla al encontrarla
                }

                // Si se han encontrado suficientes asientos, detener la búsqueda
                if asientos_recomendados.len() == cantidad_boletos as usize {
                    break;
                }
            }

            // Si se han encontrado suficientes asientos, detener la búsqueda
            if asientos_recomendados.len() == cantidad_boletos as usize {
                break;
            }
        }

        if asientos_recomendados.len() < cantidad_boletos as usize {
            mensaje = format!("No se encontraron suficientes asientos disponibles en la categoría: {}", categoria.nombre);
        }

        (asientos_recomendados, mensaje, categoria.nombre.clone())
    }

    // Función para confirmar o cancelar la compra de asientos (cambia su estado a Comprada o Disponible)
    fn confirmar_compra_sillas(&mut self, indice_categoria: usize, asientos: &mut Vec<(String, u32, Silla)>, confirmar: bool) {
        let categoria = &mut self.categorias[indice_categoria];
        for (zona_nombre, numero_fila, silla) in asientos.iter_mut() {
            // Encontrar la silla en la estructura del estadio
            if let Some(zona) = categoria.zonas.iter_mut().find(|zona| zona.nombre == *zona_nombre) {
                if let Some(fila) = zona.filas.get_mut(&numero_fila) {
                    if let Some(silla_actual) = fila.iter_mut().find(|s| s.numero == silla.numero) {
                        silla_actual.estado = if confirmar {
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