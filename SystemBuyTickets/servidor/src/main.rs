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
    filas: Vec<Silla>
}
//-------------------------------------------------
struct Categoria {
    nombre: String,         //POSIBLES NOMBRES -> 1.Tribuna Norte, 2. Tribuna Sur, 3.Platea Lateral Este, 4.Platea Lateral Oeste 
    zonas: Vec<Zona>,
}   
//-------------------------------------------------
struct Estadio {
    categorias: Vec<Categoria>,
}
//=========================================CREATED OF NECESSARY FUNCTIONS=========================================
fn main() {
    print!("Codigo Realizado Con Exito");
}
