use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;


//-------------CREACION DE LAS ESTRUCTURAS NECESARIAS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EstadoSilla {
    Disponible,
    Reservada,
    Comprada,
}

#[derive(Debug, Clone)]
struct Silla {
    numero: u32,
    estado: EstadoSilla,
}

#[derive(Debug, Clone)]
struct Zona {
    nombre: String,
    filas: HashMap<u32, Vec<Silla>>,
}

#[derive(Debug, Clone)]
struct Categoria {
    nombre: String,
    zonas: Vec<Zona>,
}

#[derive(Debug, Clone)]
struct Estadio {
    categorias: Vec<Categoria>,
}

//--------------INICIALIZACION DEL MAPEO DEL ESTADIO 
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

//--------------CREACION DEL MENU PARA VISUALIZACION DEL CLIENTE
async fn crear_menu(estadio: &Estadio) -> String {
    let mut menu = String::new();
    menu.push_str("--- Menú del Estadio ---\n");
    for (i, categoria) in estadio.categorias.iter().enumerate() {
        menu.push_str(&format!("{}. {}\n", i + 1, categoria.nombre));
    }
    menu.push_str("-----------------------\n");
    menu.push_str("Ingrese el número de la categoría para buscar asientos.\n");
    menu.push_str("Ingrese '0' para salir\n");
    menu
}

//---------------FUNCION PARA MANEJAR AL CLIENTE
async fn manejar_cliente(mut stream: TcpStream, estadio: Arc<Mutex<Estadio>>) {
    let (reader, mut writer) = io::split(stream);
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = String::new();

    loop {
        let estadio_guard = estadio.lock().await;
        let menu = crear_menu(&*estadio_guard).await;

        if let Err(e) = writer.write_all(menu.as_bytes()).await {
            eprintln!("Error al enviar el menú: {:?}", e);
            return;
        }
        if let Err(e) = writer.flush().await {
            eprintln!("Error al hacer flush: {:?}", e);
            return;
        }

        buffer.clear();
        match buf_reader.read_line(&mut buffer).await {
            Ok(bytes_read) if bytes_read > 0 => {
                let opcion = buffer.trim();
                if opcion.eq_ignore_ascii_case("0") {
                    let msg = "Conexión terminada.\n";
                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        eprintln!("Error al enviar mensaje de terminación: {:?}", e);
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Error al hacer flush: {:?}", e);
                    }
                    break;
                }

                // Mostrar el valor ingresado para diagnóstico
                println!("Opción ingresada: {}", opcion);

                let categoria_index = opcion.parse::<usize>().ok();
                match categoria_index {
                    Some(index) if index > 0 && index <= estadio_guard.categorias.len() => {
                        let categoria = &estadio_guard.categorias[index - 1];
                        println!("Categoría seleccionada: {}", categoria.nombre); // Diagnóstico

                        let resultado = estadio_guard.buscar_mejores_asientos_2(categoria.nombre.clone(), None, 4);
                        let response = match resultado {
                            Some(asientos) => format!("Asientos encontrados: {:?}\n", asientos),
                            None => "No se encontraron asientos disponibles\n".to_string(),
                        };

                        if let Err(e) = writer.write_all(response.as_bytes()).await {
                            eprintln!("Error al enviar la respuesta: {:?}", e);
                            break;
                        }
                        if let Err(e) = writer.flush().await {
                            eprintln!("Error al hacer flush: {:?}", e);
                            break;
                        }
                    }
                    _ => {
                        println!("Opción inválida o fuera de rango: {}", opcion); // Diagnóstico
                        let msg = "Opción inválida\n";
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            eprintln!("Error al enviar mensaje de opción inválida: {:?}", e);
                        }
                        if let Err(e) = writer.flush().await {
                            eprintln!("Error al hacer flush: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Ok(_) => eprintln!("No se recibió ningún dato"),
            Err(e) => {
                eprintln!("Error al leer del cliente: {:?}", e);
                break;
            }
        }
    }
}

//-------------MAIN PRINCIPAL
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let estadio = Arc::new(Mutex::new(inicializar_mapeo().await));
    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    println!("Servidor escuchando en el puerto 7878");

    loop {
        let (stream, _) = listener.accept().await?;
        let estadio = Arc::clone(&estadio);
        tokio::spawn(async move {
            manejar_cliente(stream, estadio).await;
        });
    }
}

//=======================FUNCIONES DE BUSQUEDA, MODIFICACION DE ESTADO========================
//--------------------FUNCION DE BUSQUEDA DE MEJORES SILLAS
impl Estadio {
    pub fn buscar_mejores_asientos_2(
        &self,
        categoria_nombre: String,
        zona_nombre: Option<String>,
        cantidad: u32,
    ) -> Option<Vec<(String, u32, Vec<u32>)>> {
        // Buscar la categoría correspondiente por nombre
        let categoria = self.categorias.iter().find(|c| c.nombre == categoria_nombre)?;

        let mut mejor_opcion: Option<Vec<(String, u32, Vec<u32>)>> = None;
        let mut max_asientos_juntos = 0;

        let buscar_zona = |zona: &Zona, cantidad: u32| -> Option<Vec<(String, u32, Vec<u32>)>> {
            let mut mejor_asientos_en_zona: Option<Vec<(String, u32, Vec<u32>)>> = None;

            for (fila_numero, fila) in &zona.filas {
                let mut asientos_juntos: Vec<u32> = Vec::new();

                for silla in fila {
                    if matches!(silla.estado, EstadoSilla::Disponible) {
                        asientos_juntos.push(silla.numero);
                    } else {
                        asientos_juntos.clear(); // Reiniciar si encontramos un asiento no disponible
                    }

                    if asientos_juntos.len() == cantidad as usize {
                        let asientos = asientos_juntos.clone();
                        mejor_asientos_en_zona = Some(vec![(zona.nombre.clone(), *fila_numero, asientos)]);
                        break;
                    }
                }

                if mejor_asientos_en_zona.is_some() {
                    break;
                }
            }

            mejor_asientos_en_zona
        };

        if let Some(zona_nombre) = zona_nombre {
            // Buscar en la zona específica si se proporciona
            if let Some(zona) = categoria.zonas.iter().find(|z| z.nombre == zona_nombre) {
                mejor_opcion = buscar_zona(zona, cantidad);
            }
        }

        if mejor_opcion.is_none() {
            // Buscar en todas las zonas si no se encontró en la zona específica o si no se proporcionó
            for zona in &categoria.zonas {
                if let Some(asientos_en_zona) = buscar_zona(zona, cantidad) {
                    let cantidad_juntos = asientos_en_zona.iter().map(|(_, _, asientos)| asientos.len()).sum::<usize>();

                    // Si encontramos más asientos juntos en esta zona que en la mejor opción actual, actualizamos la mejor opción
                    if cantidad_juntos > max_asientos_juntos {
                        max_asientos_juntos = cantidad_juntos;
                        mejor_opcion = Some(asientos_en_zona);
                    }
                }
            }
        }

        // Si no encontramos suficientes asientos juntos en ninguna zona, buscamos la mejor combinación posible
        if max_asientos_juntos < cantidad as usize {
            mejor_opcion = None;
            let mut asientos_totales: Vec<(String, u32, Vec<u32>)> = Vec::new();
            let mut asientos_encontrados = 0;

            for zona in &categoria.zonas {
                for (fila_numero, fila) in &zona.filas {
                    let mut asientos_juntos: Vec<u32> = Vec::new();

                    for silla in fila {
                        if matches!(silla.estado, EstadoSilla::Disponible) {
                            asientos_juntos.push(silla.numero);
                            asientos_encontrados += 1;

                            if asientos_encontrados == cantidad {
                                asientos_totales.push((zona.nombre.clone(), *fila_numero, asientos_juntos));
                                return Some(asientos_totales);
                            }
                        }

                        if asientos_juntos.len() > 0 && !matches!(silla.estado, EstadoSilla::Disponible) {
                            if !asientos_juntos.is_empty() {
                                asientos_totales.push((zona.nombre.clone(), *fila_numero, asientos_juntos.clone()));
                                asientos_juntos.clear();
                            }
                        }
                    }

                    if !asientos_juntos.is_empty() {
                        asientos_totales.push((zona.nombre.clone(), *fila_numero, asientos_juntos.clone()));
                    }

                    if asientos_encontrados >= cantidad {
                        return Some(asientos_totales);
                    }
                }
            }

            if asientos_encontrados >= cantidad {
                mejor_opcion = Some(asientos_totales);
            }
        }

        mejor_opcion
    }
}