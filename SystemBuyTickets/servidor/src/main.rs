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

    // Crear la primera categoría: Platea Este
    let mut platea_este = Categoria {
        nombre: "Platea Este".to_string(),
        zonas: Vec::new(),
    };

    // Crear zonas para Platea Este
    let mut zona_a = Zona {
        nombre: "Zona A".to_string(),
        filas: HashMap::new(),
    };
    let mut zona_b = Zona {
        nombre: "Zona B".to_string(),
        filas: HashMap::new(),
    };

    // Agregar filas y sillas a Zona A
    let fila_1_a: Vec<Silla> = (1..=10).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_a: Vec<Silla> = (1..=10).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_a.filas.insert(1, fila_1_a);
    zona_a.filas.insert(2, fila_2_a);

    // Agregar filas y sillas a Zona B
    let fila_1_b: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_b: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_b.filas.insert(1, fila_1_b);
    zona_b.filas.insert(2, fila_2_b);

    platea_este.zonas.push(zona_a);
    platea_este.zonas.push(zona_b);

    // Crear la segunda categoría: General
    let mut general = Categoria {
        nombre: "General".to_string(),
        zonas: Vec::new(),
    };

    // Crear zonas para General
    let mut zona_c = Zona {
        nombre: "Zona C".to_string(),
        filas: HashMap::new(),
    };
    let mut zona_d = Zona {
        nombre: "Zona D".to_string(),
        filas: HashMap::new(),
    };

    // Agregar filas y sillas a Zona C
    let fila_1_c: Vec<Silla> = (1..=15).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_c: Vec<Silla> = (1..=15).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_c.filas.insert(1, fila_1_c);
    zona_c.filas.insert(2, fila_2_c);

    // Agregar filas y sillas a Zona D
    let fila_1_d: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_d: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_d.filas.insert(1, fila_1_d);
    zona_d.filas.insert(2, fila_2_d);

    general.zonas.push(zona_c);
    general.zonas.push(zona_d);

    categorias.push(platea_este);
    categorias.push(general);

    Estadio {
        categorias,
    }
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

    // Obtener el estadio
    let mut estadio = estadio.lock().await;

    // Buscar asientos
    let (mut asientos_recomendados, mensaje) = estadio.buscar_asientos(indice_categoria, cantidad_boletos, 10);

    // Crear mensaje para el cliente
    let mut respuesta = format!("{}\n", mensaje);
    if !asientos_recomendados.is_empty() {
        respuesta.push_str("Asientos recomendados:\n");
        for silla in &asientos_recomendados {
            respuesta.push_str(&format!("Número: {}, Estado: {:?}\n", silla.numero, silla.estado));
        }
        // Reservar los asientos encontrados
        estadio.reservar_sillas(&mut asientos_recomendados);

        // Confirmar compra si es necesario
        if confirmar_compra {
            estadio.confirmar_compra_sillas(&mut asientos_recomendados, true);
            respuesta.push_str("\nCompra realizada.");
        } else {
            estadio.confirmar_compra_sillas(&mut asientos_recomendados, false);
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

    println!("Servidor escuchando en el puerto '7878'");

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
    fn buscar_asientos(&mut self, indice_categoria: usize, cantidad_boletos: u32, max_boletos: u32) -> (Vec<Silla>, String) {
        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string());
        }

        let categoria = &mut self.categorias[indice_categoria];
        let mut asientos_recomendados = Vec::new();
        let mut mensaje = String::from("");

        for zona in &categoria.zonas {
            for (numero_fila, asientos) in &zona.filas {
                let mut asientos_disponibles: Vec<&Silla> = asientos.iter()
                    .filter(|silla| silla.estado == EstadoSilla::Disponible)
                    .collect();
                
                if asientos_disponibles.len() >= cantidad_boletos as usize {
                    asientos_recomendados.extend(asientos_disponibles.into_iter().take(cantidad_boletos as usize).cloned());
                    mensaje = format!("Asientos encontrados en la zona '{}', fila '{}'", zona.nombre, numero_fila);
                    return (asientos_recomendados, mensaje);
                }
            }
        }

        mensaje = "No se encontraron suficientes asientos disponibles".to_string();
        (Vec::new(), mensaje)
    }

    fn reservar_sillas(&mut self, asientos: &mut Vec<Silla>) {
        for silla in asientos {
            silla.estado = EstadoSilla::Reservada;
        }
    }

    fn confirmar_compra_sillas(&mut self, asientos: &mut Vec<Silla>, confirmar: bool) {
        if confirmar {
            for silla in asientos {
                silla.estado = EstadoSilla::Comprada;
            }
        } else {
            for silla in asientos {
                silla.estado = EstadoSilla::Disponible;
            }
        }
    }
}
