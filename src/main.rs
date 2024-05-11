use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser;

/// Une 2 archivos srt en un único subtítulo mostrando uno en la parte superior y el otro en la inferior
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ruta que apunta al archivo que se situará en la parte superior
    #[arg(short, long)]
    superior: PathBuf,

    /// ruta que apunta al archivo que se situará en la parte inferior
    #[arg(short, long)]
    inferior: PathBuf,
}
#[derive(Clone, PartialEq)]
enum Posicion {
    Superior, // {\an8}
    Inferior, //  {\an2}
}
#[derive(Clone)]
struct IndicadorTemporal {
    inicio: u64,
    fin: u64,
}

struct Subtitulo {
    tiempo: IndicadorTemporal,
    contenido: String,
    posicion: Posicion,
}

/// Devuelve un indicador temporal en milisegundos: 00:00:02,049 => 2049
fn indicador_temporal_a_milisegundos(indicador: &str) -> Result<u64, String> {
    // Primera parte, comprobar milisegundos
    let hh_mm_ss_mmm = indicador.split(",").collect::<Vec<&str>>();
    if hh_mm_ss_mmm.len() != 2 {
        return Err("el divisor de milisegundos , no ha devuelto 2 resultados".to_string());
    }
    let milisegundos;

    match hh_mm_ss_mmm[1].parse::<u64>() {
        Ok(ok) => milisegundos = ok,
        Err(error) => {
            let mensaje_error = format!(
                "no se ha podido obtener el número de milisegundos {}",
                error
            );
            return Err(mensaje_error);
        }
    }

    // Segunda parte, comprobar hh:mm:ss
    let hh_mm_ss = hh_mm_ss_mmm[0].split(":").collect::<Vec<&str>>();
    if hh_mm_ss.len() != 3 {
        return Err("el divisor de hh:mm:ss (:) no ha devuelto 3 resultados".to_string());
    }

    let horas;
    match hh_mm_ss[0].parse::<u64>() {
        Ok(ok) => horas = ok,
        Err(error) => {
            let mensaje_error = format!("no se ha podido obtener el número de horas {}", error);
            return Err(mensaje_error);
        }
    }
    let horas_milisegundos = horas * 3600000;

    let minutos;
    match hh_mm_ss[1].parse::<u64>() {
        Ok(ok) => minutos = ok,
        Err(error) => {
            let mensaje_error = format!("no se ha podido obtener el número de minutos {}", error);
            return Err(mensaje_error);
        }
    }
    let minutos_milisegundos = minutos * 60000;

    let segundos;
    match hh_mm_ss[2].parse::<u64>() {
        Ok(ok) => segundos = ok,
        Err(error) => {
            let mensaje_error = format!("no se ha podido obtener el número de segundos {}", error);
            return Err(mensaje_error);
        }
    }
    let segundos_milisegundos = segundos * 1000;

    Ok(horas_milisegundos + minutos_milisegundos + segundos_milisegundos + milisegundos)
}

/// Determina si una línea es un indicador temporal y devuelve el inicio y el final en milisegundos
fn parsear_linea_temporal(linea: &str) -> Result<IndicadorTemporal, String> {
    // Comprobar existe inicio y fin
    let inicio_fin = linea.split(" --> ").collect::<Vec<&str>>();
    if inicio_fin.len() != 2 {
        return Err("el divisor --> no ha devuelto un inicio y un fin".to_string());
    }

    let inicio;
    match indicador_temporal_a_milisegundos(inicio_fin[0]) {
        Ok(ok) => inicio = ok,
        Err(error) => {
            let mensaje_error = format!(
                "no se ha podido obtener el indicador temporal del inicio {}",
                error
            );
            return Err(mensaje_error);
        }
    }

    let fin;
    match indicador_temporal_a_milisegundos(inicio_fin[1]) {
        Ok(ok) => fin = ok,
        Err(error) => {
            let mensaje_error = format!(
                "no se ha podido obtener el indicador temporal del fin {}",
                error
            );
            return Err(mensaje_error);
        }
    }

    Ok(IndicadorTemporal { inicio, fin })
}

/// Determina si la línea es identificador numérico. Condiciones: línea u64 y linea_siguiente IndicadorTemporal
fn es_identificador_numero(linea: &str, linea_siguiente: &str) -> bool {
    match linea.parse::<u64>() {
        Ok(_) => (),
        Err(_) => return false,
    }

    match parsear_linea_temporal(linea_siguiente) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

fn parser_archivo_srt(ruta: PathBuf, posicion: Posicion) -> Result<Vec<Subtitulo>, String> {
    /*
        Formato SRT:
    1
    00:00:02,049 --> 00:00:03,264
    Hola mundo 1

    2
    00:00:03,264 --> 00:00:05,724
    Hola mundo 2

    */

    let mut archivo;
    match File::open(ruta.clone()) {
        Ok(ok) => archivo = ok,
        Err(error) => {
            let mensaje_error = format!("no se ha podido abrir el archivo {:?}: {}", ruta, error);
            return Err(mensaje_error);
        }
    }
    let mut srt = String::new();
    match archivo.read_to_string(&mut srt) {
        Ok(_) => (),
        Err(error) => {
            let mensaje_error = format!(
                "no se ha podido leer el contenido del archivo {:?}: {}",
                ruta, error
            );
            return Err(mensaje_error);
        }
    }

    let mut subtitulos = Vec::new();

    let mut contenido_subtitulo = String::new();
    let mut indicador_temporal = IndicadorTemporal { inicio: 0, fin: 0 };

    let lineas_srt = srt.lines().collect::<Vec<&str>>();
    for indice in 0..lineas_srt.len() {
        let linea = lineas_srt[indice];
        let mut linea_siguiente = "";
        if indice < lineas_srt.len() - 1 {
            linea_siguiente = lineas_srt[indice + 1];
        }

        /* Cuando se encuentra un número se incorpora y se borra el contenido */
        let es_numero = es_identificador_numero(linea, linea_siguiente);
        if es_numero {
            // Excepción para el primer número que no tiene contenido que incorporar
            if contenido_subtitulo != "" {
                let subtitulo = Subtitulo {
                    tiempo: indicador_temporal.clone(),
                    contenido: contenido_subtitulo.clone(),
                    posicion: posicion.clone(),
                };
                subtitulos.push(subtitulo);
                contenido_subtitulo = String::from("");
            }

            continue;
        }

        // ¿Es indicador temporal?
        match parsear_linea_temporal(linea) {
            Ok(ok) => {
                indicador_temporal = ok;
                continue;
            }
            Err(_) => (),
        }

        // No es identificador ni indicador temporal, es contenido del subtítulo

        contenido_subtitulo.push_str(linea);
    }

    // Hay que incorporar el último que queda descolgado ya que el push se hace desde un identificador
    let subtitulo = Subtitulo {
        tiempo: indicador_temporal.clone(),
        contenido: contenido_subtitulo.clone(),
        posicion: posicion.clone(),
    };
    subtitulos.push(subtitulo);

    Ok(subtitulos)
}

/// Convierte milisegundos a marca temporal. 2049 a 00:00:02,049
fn milisegundos_a_marca_temporal(milisegundos: u64) -> String {
    let horas = milisegundos / 3600000;
    let minutos = (milisegundos % 3600000) / 60000;
    let segundos = ((milisegundos % 3600000) % 60000) / 1000;
    let mili = ((milisegundos % 3600000) % 60000) % 1000;

    format!("{:02}:{:02}:{:02},{:03}", horas, minutos, segundos, mili)
}
/// Convierte un indicador temporal (inicio u64, fin u64) en marca temporal 00:00:02,049 --> 00:00:03,264
fn marca_temporal_desde_indicador_temporal(indicador: &IndicadorTemporal) -> String {
    format!(
        "{} --> {}",
        milisegundos_a_marca_temporal(indicador.inicio),
        milisegundos_a_marca_temporal(indicador.fin)
    )
}

/// Prepara el contenido y escribe subtitulos_superior_inferior.srt
fn unir_escribir_archivo(mut subtitulos: Vec<Subtitulo>) {
    // Ordenar Vec por inicio
    subtitulos.sort_by_key(|s| s.tiempo.inicio);

    let mut subtitulo_final = String::new();
    for i in 0..subtitulos.len() {
        let subtitulo = &subtitulos[i];
        let identificador_subtitulo = i + 1;
        let codigo_posicion;
        if subtitulo.posicion == Posicion::Superior {
            codigo_posicion = r"{\an8}";
        } else {
            codigo_posicion = r"{\an2}";
        }
        let marca_temporal = marca_temporal_desde_indicador_temporal(&subtitulo.tiempo);
        let s = format!(
            "{}
{}
{}{}

",
            identificador_subtitulo, marca_temporal, codigo_posicion, subtitulo.contenido
        );
        subtitulo_final.push_str(&s);
    }

    let mut archivo_destino;
    match File::create("subtitulos_superior_inferior.srt") {
        Ok(ok) => archivo_destino = ok,
        Err(error) => {
            panic!(
                "ERROR CRÍTICO: no se ha podido crear el archivo para los subtítulos combinados {}",
                error
            )
        }
    }

    match archivo_destino.write(subtitulo_final.as_bytes()) {
        Ok(_) => {
            println!("ÉXITO: archivo subtitulos_superior_inferior.srt creado satisfactoriamente")
        }
        Err(error) => {
            panic!(
                "ERROR CRÍTICO: no se ha podido escribir los subtítulos combinados {}",
                error
            )
        }
    }
}

fn main() {
    let argumentos = Args::parse();

    let subtitulos_superior;
    match parser_archivo_srt(argumentos.superior, Posicion::Superior) {
        Ok(ok) => subtitulos_superior = ok,
        Err(error) => {
            panic!(
                "ERROR CRÍTICO: no se ha podido procesar el archivo para la parte superior: {}",
                error
            )
        }
    }

    let subtitulos_inferior;
    match parser_archivo_srt(argumentos.inferior, Posicion::Inferior) {
        Ok(ok) => subtitulos_inferior = ok,
        Err(error) => {
            panic!("ERROR CRÍTICO: no se ha podido procesar el archivo con los subtítulos para la parte inferior: {}", error)
        }
    }

    let mut total_subtitulos = Vec::new();
    for v in subtitulos_superior {
        total_subtitulos.push(v)
    }
    for v in subtitulos_inferior {
        total_subtitulos.push(v)
    }

    if total_subtitulos.len() == 0 {
        panic!("ERROR CRÍTICO: no hay subtítulos, no hay nada que añadir al nuevo archivo")
    }

    unir_escribir_archivo(total_subtitulos);
}
