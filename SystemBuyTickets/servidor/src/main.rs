// servidor.rs

use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

//------------------------------------Estructuras necesarias
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoSilla {
    Disponible,
    Reservada,
    Comprada,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Silla {
    pub numero: u32,
    pub estado: EstadoSilla,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zona {
    pub nombre: String,
    pub filas: HashMap<u32, Vec<Silla>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Categoria {
    pub nombre: String,
    pub zonas: Vec<Zona>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estadio {
    pub categorias: Vec<Categoria>,
}

// Estructuras para la comunicación
#[derive(Debug, Serialize, Deserialize)]
struct Solicitud {
    indice_categoria: usize,
    cantidad_boletos: u32,
    confirmar_compra: bool,
    asientos_recomendados: Option<Vec<AsientoInfoCliente>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RespuestaServidor {
    categoria: String,
    mensaje: String,
    asientos_categoria: Vec<AsientoInfo>,
    asientos_recomendados: Vec<AsientoInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsientoInfo {
    zona: String,
    fila: u32,
    asiento: u32,
    estado: EstadoSilla,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsientoInfoCliente {
    zona: String,
    fila: u32,
    asiento: u32,
    // No incluimos 'estado' aquí
}

//-------------------------------Inicializar el mapeo del estadio
async fn inicializar_mapeo() -> Estadio {
    let mut categorias = Vec::new();

    // Se define un vector de nombres de categorías.
    let nombres_categorias = vec!["Platea Este", "Platea Oeste", "General Norte", "General Sur"];
    
    // Se itera sobre cada nombre de categoría.
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

//=======================FUNCIONES DE BUSQUEDA, MODIFICACION DE ESTADO========================
impl Estadio {
    // Función para obtener todos los asientos de una categoría
    fn obtener_asientos_categoria(&self, indice_categoria: usize) -> (Vec<AsientoInfo>, String) {
        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string());
        }

        let categoria = &self.categorias[indice_categoria];
        let mut asientos_info = Vec::new();

        for zona in &categoria.zonas {
            for (numero_fila, asientos) in &zona.filas {
                for silla in asientos {
                    asientos_info.push(AsientoInfo {
                        zona: zona.nombre.clone(),
                        fila: *numero_fila,
                        asiento: silla.numero,
                        estado: silla.estado,
                    });
                }
            }
        }

        (asientos_info, categoria.nombre.clone())
    }

    // Función para buscar los mejores asientos disponibles en una categoría
    fn buscar_asientos(&mut self, indice_categoria: usize, cantidad_boletos: u32, max_boletos: u32) -> (Vec<AsientoInfo>, String) {
        // Verificación de la cantidad de boletos permitidos
        if cantidad_boletos > max_boletos {
            return (Vec::new(), "Transacción no permitida: Excede el máximo de asientos permitidos para comprar".to_string());
        }

        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string());
        }

        let categoria = &mut self.categorias[indice_categoria];
        let mut asientos_recomendados = Vec::new();
        let mut mensaje = String::from("");

        // Itera sobre las zonas y filas de la categoría seleccionada
        for zona in &mut categoria.zonas {
            for (numero_fila, asientos) in &mut zona.filas {
                let asientos_disponibles: Vec<&mut Silla> = asientos.iter_mut()
                    .filter(|silla| silla.estado == EstadoSilla::Disponible)
                    .collect();

                // Añadir asientos hasta completar la cantidad solicitada
                for silla in asientos_disponibles.into_iter().take(cantidad_boletos as usize - asientos_recomendados.len()) {
                    silla.estado = EstadoSilla::Reservada; // Actualiza el estado de la silla al reservarla
                    asientos_recomendados.push(AsientoInfo {
                        zona: zona.nombre.clone(),
                        fila: *numero_fila,
                        asiento: silla.numero,
                        estado: silla.estado,
                    });
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
            mensaje = format!("No se encontraron suficientes asientos disponibles en la categoría.");
        }

        (asientos_recomendados, mensaje)
    }

    // Función para confirmar o cancelar la compra de asientos (cambia su estado a Comprada o Disponible)
    fn confirmar_compra_sillas(&mut self, indice_categoria: usize, asientos: &Vec<AsientoInfoCliente>, confirmar: bool) {
        let categoria = &mut self.categorias[indice_categoria];
        for asiento_info in asientos {
            if let Some(zona) = categoria.zonas.iter_mut().find(|zona| zona.nombre == asiento_info.zona) {
                if let Some(fila) = zona.filas.get_mut(&asiento_info.fila) {
                    if let Some(silla_actual) = fila.iter_mut().find(|s| s.numero == asiento_info.asiento) {
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

//----------------------Función para deserializar y validar la solicitud
fn deserializar_solicitud(datos: &str) -> Result<Solicitud, String> {
    serde_json::from_str(datos).map_err(|e| e.to_string())
}

// Función para manejar al cliente
async fn manejar_cliente(mut stream: TcpStream, estadio: Arc<Mutex<Estadio>>) {
    // Leer los datos del cliente
    let mut buffer = [0; 65536];
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
            let respuesta = RespuestaServidor {
                categoria: "".to_string(),
                mensaje: format!("Error al deserializar solicitud: {}", e),
                asientos_categoria: Vec::new(),
                asientos_recomendados: Vec::new(),
            };
            let respuesta_json = serde_json::to_string(&respuesta).unwrap();
            if let Err(e) = stream.write_all(respuesta_json.as_bytes()).await {
                eprintln!("Error al escribir al stream: {:?}", e);
            }
            return;
        }
    };

    let Solicitud {
        indice_categoria,
        cantidad_boletos,
        confirmar_compra,
        asientos_recomendados,
    } = solicitud;

    // Bloquear el estadio para modificarlo
    let mut estadio = estadio.lock().await;

    // Obtener todos los asientos de la categoría seleccionada
    let (asientos_categoria, nombre_categoria) = estadio.obtener_asientos_categoria(indice_categoria);

    let mut respuesta = RespuestaServidor {
        categoria: nombre_categoria.clone(),
        mensaje: "".to_string(),
        asientos_categoria: asientos_categoria.clone(),
        asientos_recomendados: Vec::new(),
    };

    if !confirmar_compra && asientos_recomendados.is_none() {
        // Primera solicitud: Buscar asientos y marcarlos como reservados
        let (asientos_recomendados, mensaje_busqueda) = estadio.buscar_asientos(indice_categoria, cantidad_boletos, 10);
        respuesta.asientos_recomendados = asientos_recomendados.clone();
        respuesta.mensaje = mensaje_busqueda;
    } else {
        // Segunda solicitud: Confirmar o cancelar compra
        if let Some(asientos) = asientos_recomendados {
            estadio.confirmar_compra_sillas(indice_categoria, &asientos, confirmar_compra);
            respuesta.mensaje = if confirmar_compra {
                "Compra realizada.".to_string()
            } else {
                "Compra cancelada. Asientos liberados.".to_string()
            };
        } else {
            respuesta.mensaje = "No se proporcionaron asientos para confirmar o cancelar.".to_string();
        }
    }

    // Actualizar la lista de asientos después de las operaciones
    let (asientos_actualizados, _) = estadio.obtener_asientos_categoria(indice_categoria);
    respuesta.asientos_categoria = asientos_actualizados;

    // Serializar y enviar la respuesta al cliente
    let respuesta_json = serde_json::to_string(&respuesta).unwrap();
    if let Err(e) = stream.write_all(respuesta_json.as_bytes()).await {
        eprintln!("Error al escribir al stream: {:?}", e);
    }
}

//-----------------MAIN PRINCIPAL
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
