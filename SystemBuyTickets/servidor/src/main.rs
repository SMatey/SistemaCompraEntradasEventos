use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;


//-------------CREACION DE LAS ESTRUCTURAS NECESARIAS
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
pub struct Fila {
    pub numero: u32,
    pub asientos: Vec<Silla>,
}

#[derive(Debug, Clone)]
pub struct Zona {
    pub nombre: String,
    pub filas: HashMap<u32, Fila>,
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
    zona_a.filas.insert(1, Fila { numero: 1, asientos: fila_1_a });
    zona_a.filas.insert(2, Fila { numero: 2, asientos: fila_2_a });

    // Agregar filas y sillas a Zona B
    let fila_1_b: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_b: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_b.filas.insert(1, Fila { numero: 1, asientos: fila_1_b });
    zona_b.filas.insert(2, Fila { numero: 2, asientos: fila_2_b });

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
    zona_c.filas.insert(1, Fila { numero: 1, asientos: fila_1_c });
    zona_c.filas.insert(2, Fila { numero: 2, asientos: fila_2_c });

    // Agregar filas y sillas a Zona D
    let fila_1_d: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    let fila_2_d: Vec<Silla> = (1..=12).map(|num| Silla {
        numero: num,
        estado: EstadoSilla::Disponible,
    }).collect();
    zona_d.filas.insert(1, Fila { numero: 1, asientos: fila_1_d });
    zona_d.filas.insert(2, Fila { numero: 2, asientos: fila_2_d });

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

        // Enviar el menú al cliente
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
                println!("Opción ingresada: {}", opcion); // Debugging

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

                let categoria_index = opcion.parse::<usize>().ok();
                if let Some(index) = categoria_index {
                    let categoria = &estadio_guard.categorias.get(index - 1);
                    if let Some(categoria) = categoria {
                        // Enviar confirmación de categoría
                        let msg = format!("Categoría seleccionada: {}\n", categoria.nombre);
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            eprintln!("Error al enviar confirmación de categoría: {:?}", e);
                            break;
                        }
                        if let Err(e) = writer.flush().await {
                            eprintln!("Error al hacer flush: {:?}", e);
                            break;
                        }

                        // Leer la cantidad de asientos
                        buffer.clear();
                        match buf_reader.read_line(&mut buffer).await {
                            Ok(bytes_read) if bytes_read > 0 => {
                                let cantidad = buffer.trim().parse::<u32>().unwrap_or(0);
                                let max_boletos = 6; // Definir límite máximo de boletos

                                if cantidad > max_boletos {
                                    let msg = "La cantidad solicitada excede el límite máximo de boletos.\n";
                                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                                        eprintln!("Error al enviar mensaje de límite excedido: {:?}", e);
                                    }
                                    if let Err(e) = writer.flush().await {
                                        eprintln!("Error al hacer flush: {:?}", e);
                                    }
                                } else {
                                    let resultado = estadio_guard.buscar_mejores_asientos_2(categoria.nombre.clone(), cantidad, max_boletos);
                                    let response = match resultado {
                                        Some(asientos) => format!("Asientos encontrados: {:?}\n", asientos),
                                        None => "No se encontraron asientos disponibles\n".to_string(),
                                    };

                                    // Enviar la respuesta al cliente
                                    if let Err(e) = writer.write_all(response.as_bytes()).await {
                                        eprintln!("Error al enviar la respuesta: {:?}", e);
                                        break;
                                    }
                                    if let Err(e) = writer.flush().await {
                                        eprintln!("Error al hacer flush: {:?}", e);
                                        break;
                                    }
                                }
                            }
                            Ok(_) => eprintln!("No se recibió ningún dato para la cantidad"),
                            Err(e) => {
                                eprintln!("Error al leer la cantidad de asientos: {:?}", e);
                                break;
                            }
                        }
                    } else {
                        let msg = "Categoría no válida.\n";
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            eprintln!("Error al enviar mensaje de categoría no válida: {:?}", e);
                        }
                        if let Err(e) = writer.flush().await {
                            eprintln!("Error al hacer flush: {:?}", e);
                        }
                    }
                } else {
                    let msg = "Opción inválida.\n";
                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        eprintln!("Error al enviar mensaje de opción inválida: {:?}", e);
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Error al hacer flush: {:?}", e);
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
    pub fn reservar_asientos(
        &mut self,
        categoria_nombre: &str,
        zona_nombre: &str,
        fila_numero: u32,
        asientos: Vec<u32>,
    ) -> Result<(), String> {
        // Buscar la categoría correspondiente por nombre
        let categoria = self.categorias.iter_mut().find(|c| c.nombre == categoria_nombre)
            .ok_or("Categoría no encontrada")?;
        
        // Buscar la zona correspondiente por nombre
        let zona = categoria.zonas.iter_mut().find(|z| z.nombre == zona_nombre)
            .ok_or("Zona no encontrada")?;
        
        // Buscar la fila correspondiente por número
        let fila = zona.filas.get_mut(&fila_numero)
            .ok_or("Fila no encontrada")?;
        
        // Cambiar el estado de los asientos a Reservada
        for numero in asientos {
            let silla = fila.asientos.iter_mut().find(|s| s.numero == numero)
                .ok_or("Asiento no encontrado")?;
            
            if silla.estado == EstadoSilla::Disponible {
                silla.estado = EstadoSilla::Reservada;
            } else {
                return Err("El asiento no está disponible para reserva".to_string());
            }
        }

        Ok(())
    }

    pub fn comprar_asientos(
        &mut self,
        categoria_nombre: &str,
        zona_nombre: &str,
        fila_numero: u32,
        asientos: Vec<u32>,
    ) -> Result<(), String> {
        // Buscar la categoría correspondiente por nombre
        let categoria = self.categorias.iter_mut().find(|c| c.nombre == categoria_nombre)
            .ok_or("Categoría no encontrada")?;
        
        // Buscar la zona correspondiente por nombre
        let zona = categoria.zonas.iter_mut().find(|z| z.nombre == zona_nombre)
            .ok_or("Zona no encontrada")?;
        
        // Buscar la fila correspondiente por número
        let fila = zona.filas.get_mut(&fila_numero)
            .ok_or("Fila no encontrada")?;
        
        // Cambiar el estado de los asientos a Comprada
        for numero in asientos {
            let silla = fila.asientos.iter_mut().find(|s| s.numero == numero)
                .ok_or("Asiento no encontrado")?;
            
            if silla.estado == EstadoSilla::Reservada {
                silla.estado = EstadoSilla::Comprada;
            } else {
                return Err("El asiento no está reservado para compra".to_string());
            }
        }

        Ok(())
    }

    pub fn cancelar_reserva(
        &mut self,
        categoria_nombre: &str,
        zona_nombre: &str,
        fila_numero: u32,
        asientos: Vec<u32>,
    ) -> Result<(), String> {
        // Buscar la categoría correspondiente por nombre
        let categoria = self.categorias.iter_mut().find(|c| c.nombre == categoria_nombre)
            .ok_or("Categoría no encontrada")?;
        
        // Buscar la zona correspondiente por nombre
        let zona = categoria.zonas.iter_mut().find(|z| z.nombre == zona_nombre)
            .ok_or("Zona no encontrada")?;
        
        // Buscar la fila correspondiente por número
        let fila = zona.filas.get_mut(&fila_numero)
            .ok_or("Fila no encontrada")?;
        
        // Cambiar el estado de los asientos a Disponible
        for numero in asientos {
            let silla = fila.asientos.iter_mut().find(|s| s.numero == numero)
                .ok_or("Asiento no encontrado")?;
            
            if silla.estado == EstadoSilla::Reservada {
                silla.estado = EstadoSilla::Disponible;
            } else {
                return Err("El asiento no está reservado".to_string());
            }
        }

        Ok(())
    }

    pub fn buscar_mejores_asientos_2(
        &self,
        categoria_nombre: String,
        cantidad: u32,
        max_boletos: u32,
    ) -> Option<Vec<(String, u32, Vec<u32>)>> {
        // Buscar la categoría correspondiente por nombre
        let categoria = self.categorias.iter().find(|c| c.nombre == categoria_nombre)?;
    
        if cantidad > max_boletos {
            println!("La cantidad solicitada excede el límite máximo de boletos.");
            return None;
        }
    
        let mut mejor_opcion: Option<Vec<(String, u32, Vec<u32>)>> = None;
        let mut max_asientos_juntos = 0;
    
        for zona in &categoria.zonas {
            let mut mejor_asientos_en_zona: Option<Vec<(String, u32, Vec<u32>)>> = None;
    
            for (fila_numero, fila) in &zona.filas {
                let mut asientos_juntos: Vec<u32> = Vec::new();
                let mut asientos_proximos: Vec<u32> = Vec::new();
    
                // Iterar sobre los asientos en la fila
                for silla in fila.asientos.iter() {
                    if matches!(silla.estado, EstadoSilla::Disponible) {
                        asientos_juntos.push(silla.numero);
                        if asientos_juntos.len() == cantidad as usize {
                            let asientos = asientos_juntos.clone();
                            mejor_asientos_en_zona = Some(vec![(zona.nombre.clone(), *fila_numero, asientos)]);
                            break;
                        }
                    } else {
                        // Tratar de añadir asientos no disponibles a los asientos próximos
                        if !asientos_juntos.is_empty() {
                            asientos_proximos.push(silla.numero);
                            if asientos_proximos.len() > 0 && asientos_juntos.len() >= 1 {
                                // Agregar los asientos encontrados en la misma fila
                                asientos_proximos.extend(asientos_juntos.clone());
                                asientos_juntos.clear();
                            }
                        }
                    }
                }
    
                if mejor_asientos_en_zona.is_some() {
                    break;
                }
            }
    
            if let Some(asientos_en_zona) = mejor_asientos_en_zona {
                let cantidad_juntos = asientos_en_zona.iter().map(|(_, _, asientos)| asientos.len()).sum::<usize>();
    
                // Si encontramos más asientos juntos en esta zona que en la mejor opción actual, actualizamos la mejor opción
                if cantidad_juntos > max_asientos_juntos {
                    max_asientos_juntos = cantidad_juntos;
                    mejor_opcion = Some(asientos_en_zona);
                }
            }
        }
    
        // Si no encontramos suficientes asientos juntos, buscar la mejor combinación posible
        if max_asientos_juntos < cantidad as usize {
            mejor_opcion = None;
            let mut asientos_totales: Vec<(String, u32, Vec<u32>)> = Vec::new();
            let mut asientos_encontrados = 0;
    
            for zona in &categoria.zonas {
                for (fila_numero, fila) in &zona.filas {
                    let mut asientos_juntos: Vec<u32> = Vec::new();
    
                    // Iterar sobre los asientos en la fila
                    for silla in fila.asientos.iter() {
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
