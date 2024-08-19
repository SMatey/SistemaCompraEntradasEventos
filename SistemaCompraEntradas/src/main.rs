use std::collections::HashMap;

//=========================================CREATED OF NECESSARY STRUCTURES=========================================
enum EstadoSilla {
    Disponible,
    Reservada,
    Comprada,
}
//-------------------------------------------------
struct Silla {
    numero: u32,
    estado: EstadoSilla,
}
//-------------------------------------------------
struct  Zona {
    nombre: String,
    filas: HashMap<u32, Vec<Silla>>
}
//-------------------------------------------------
struct Categoria {
    nombre: String,
    zonas: HashMap<String, Zona>,
}   
//-------------------------------------------------
struct Estadio {
    categorias: Categoria,
}

fn main() {
    print!("Codigo Realizado Con Exito");
}
