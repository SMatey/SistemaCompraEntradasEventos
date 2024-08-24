use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

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

async fn crear_menu(estadio: &Estadio) -> String {
    let mut menu = String::new();
    menu.push_str("--- Menú del Estadio ---\n");
    for (i, categoria) in estadio.categorias.iter().enumerate() {
        menu.push_str(&format!("{}. {}\n", i + 1, categoria.nombre));
        for (j, zona) in categoria.zonas.iter().enumerate() {
            menu.push_str(&format!("   {}. {}\n", j + 1, zona.nombre));
        }
    }
    menu.push_str("-----------------------\n");
    menu.push_str("Ingrese '0' para salir");
    menu
}

async fn manejar_cliente(mut stream: TcpStream, estadio: Arc<Mutex<Estadio>>) {
    let (reader, mut writer) = io::split(stream);
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = String::new();

    let estadio = estadio.lock().await;
    let menu = crear_menu(&*estadio).await;

    loop {
        // Enviar el menú al cliente
        if let Err(e) = writer.write_all(menu.as_bytes()).await {
            eprintln!("Error al enviar el menú: {:?}", e);
            return;
        }
        if let Err(e) = writer.flush().await {
            eprintln!("Error al hacer flush: {:?}", e);
            return;
        }

        // Leer la opción del cliente
        buffer.clear();
        match buf_reader.read_line(&mut buffer).await {
            Ok(bytes_read) if bytes_read > 0 => {
                let opcion = buffer.trim();
                if opcion.eq_ignore_ascii_case("salir") {
                    let msg = "Conexión terminada.\n";
                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        eprintln!("Error al enviar mensaje de terminación: {:?}", e);
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Error al hacer flush: {:?}", e);
                    }
                    break;
                }

                let response = match opcion.parse::<usize>() {
                    Ok(valor) => {
                        if valor > 0 && valor <= estadio.categorias.len() {
                            format!("Opción {} seleccionada\n", valor)
                        } else {
                            "Opción inválida\n".to_string()
                        }
                    }
                    Err(_) => "Error al interpretar la opción\n".to_string(),
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
            Ok(_) => eprintln!("No se recibió ningún dato"),
            Err(e) => {
                eprintln!("Error al leer del cliente: {:?}", e);
                break;
            }
        }
    }
}

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
