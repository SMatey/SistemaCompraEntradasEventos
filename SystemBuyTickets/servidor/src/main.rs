use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub filas: Vec<Fila>,
}

#[derive(Debug, Clone)]
pub struct Fila {
    pub numero: u32,
    pub asientos: Vec<Silla>,
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
                filas: Vec::new(),
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
            
            // Crear filas y agregar a la zona
            let fila_1_obj = Fila { numero: 1, asientos: fila_1 };
            let fila_2_obj = Fila { numero: 2, asientos: fila_2 };
            
            zona.filas.push(fila_1_obj);
            zona.filas.push(fila_2_obj);

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

    // Verifica si el índice de categoría es válido
    if indice_categoria >= estadio.categorias.len() {
        let respuesta = format!("Categoría no válida");
        if let Err(e) = stream.write_all(respuesta.as_bytes()).await {
            eprintln!("Error al escribir al stream: {:?}", e);
        }
        return;
    }

    // Obtener el nombre de la categoría
    let nombre_categoria = estadio.categorias[indice_categoria].nombre.clone();

    // Buscar los mejores asientos disponibles
    let (mut asientos_recomendados, mensaje) = estadio.buscar_asientos(indice_categoria, cantidad_boletos, 10);

    // Crear mensaje de respuesta para el cliente
    let mut respuesta = format!("Categoría: {}\n{}\n", nombre_categoria, mensaje);
    if !asientos_recomendados.is_empty() {
        respuesta.push_str("Asientos recomendados:\n");
        for silla in &asientos_recomendados {
            // Buscar la zona y la fila de cada asiento
            let (zona_nombre, fila_numero) = estadio
                .categorias[indice_categoria]
                .zonas
                .iter()
                .find_map(|zona| {
                    zona.filas.iter().find_map(|fila| {
                        fila.asientos.iter().find_map(|s| {
                            if s.numero == silla.numero {
                                Some((zona.nombre.clone(), fila.numero))
                            } else {
                                None
                            }
                        })
                    })
                })
                .unwrap_or(("Desconocida".to_string(), 0));

            respuesta.push_str(&format!(
                "Zona: {}, Fila: {}, Asiento: {}, Estado: {:?}\n",
                zona_nombre, fila_numero, silla.numero, silla.estado
            ));
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
    let listener = TcpListener::bind("127.0.0.1:7878")
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
    // Función para buscar los mejores asientos disponibles en una categoría
    pub fn buscar_asientos(
        &mut self,
        indice_categoria: usize,
        cantidad_boletos: u32,
        max_boletos: u32,
    ) -> (Vec<Silla>, String) {
        if indice_categoria >= self.categorias.len() {
            return (Vec::new(), "Categoría no válida".to_string());
        }

        // Verifica si la cantidad de boletos solicitados supera el máximo permitido
        if cantidad_boletos > max_boletos {
            return (
                Vec::new(),
                format!(
                    "La cantidad de boletos solicitados ({}) supera el máximo permitido ({})",
                    cantidad_boletos, max_boletos
                ),
            );
        }

        let categoria = &mut self.categorias[indice_categoria];
        let mut asientos_temp = Vec::new(); // Aquí es donde acumularemos los asientos

        // Iterar sobre las zonas de la categoría seleccionada
        for zona in &mut categoria.zonas {
            // Iterar sobre las filas de la zona
            for fila in &mut zona.filas {
                // Filtrar los asientos disponibles en la fila actual
                let mut asientos_disponibles: Vec<&mut Silla> = fila
                    .asientos
                    .iter_mut()
                    .filter(|silla| silla.estado == EstadoSilla::Disponible)
                    .collect();

                // Buscar asientos juntos en la fila actual
                let mut encontrados: Vec<Silla> = Vec::new();
                let mut cont = 0;

                while cont < asientos_disponibles.len() {
                    let mut grupo = Vec::new();

                    // Buscar grupo de asientos juntos
                    for i in cont..asientos_disponibles.len() {
                        if grupo.len() < cantidad_boletos as usize {
                            grupo.push(asientos_disponibles[i].clone()); // Clonamos el asiento aquí
                        } else {
                            break;
                        }
                    }

                    if grupo.len() == cantidad_boletos as usize {
                        asientos_temp.extend(grupo);
                        break;
                    }

                    // Mover al siguiente grupo
                    cont += 1;
                }

                // Si se encontraron suficientes asientos, salir del loop de filas
                if asientos_temp.len() >= cantidad_boletos as usize {
                    break;
                }
            }

            // Si ya se han encontrado suficientes asientos, salir del loop de zonas
            if asientos_temp.len() >= cantidad_boletos as usize {
                break;
            }
        }

        // Si se encontraron suficientes asientos
        if asientos_temp.len() >= cantidad_boletos as usize {
            let asientos_recomendados: Vec<_> = asientos_temp
                .iter()
                .cloned()
                .collect();
            let mensaje = format!(
                "Asientos encontrados en la categoría '{}'.",
                categoria.nombre
            );
            self.reservar_sillas(&mut asientos_recomendados.clone());
            return (asientos_recomendados, mensaje);
        }

        // Si no se encontraron suficientes asientos
        let mensaje = "No se encontraron suficientes asientos disponibles".to_string();
        (Vec::new(), mensaje)
    }

    // Función para reservar asientos (cambia su estado a Reservada)
    pub fn reservar_sillas(&mut self, asientos: &mut Vec<Silla>) {
        // Iterar sobre cada categoría en el estadio
        for categoria in &mut self.categorias {
            // Iterar sobre cada zona en la categoría
            for zona in &mut categoria.zonas {
                // Iterar sobre cada fila en la zona
                for fila in &mut zona.filas {
                    // Iterar sobre cada silla en la fila
                    for silla in &mut fila.asientos {
                        // Si la silla está en la lista de asientos a reservar, cambiar su estado
                        if asientos.iter().any(|a| a.numero == silla.numero) {
                            silla.estado = EstadoSilla::Reservada;
                        }
                    }
                }
            }
        }
    }

    // Función para confirmar o cancelar la compra de asientos
    pub fn confirmar_compra_sillas(&mut self, indice_categoria: usize, asientos: &mut Vec<Silla>, confirmar: bool) {
        if indice_categoria >= self.categorias.len() {
            eprintln!("Categoría no válida");
            return;
        }

        let categoria = &mut self.categorias[indice_categoria];

        // Iterar sobre las zonas en la categoría
        for zona in &mut categoria.zonas {
            // Iterar sobre las filas en cada zona
            for fila in &mut zona.filas {
                // Iterar sobre los asientos en cada fila
                for silla in &mut fila.asientos {
                    // Verificar si el número de la silla está en la lista de asientos a confirmar
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

