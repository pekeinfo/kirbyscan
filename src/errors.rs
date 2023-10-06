use std::fmt;

// Define todos los posibles errores que tu aplicación podría encontrar.
#[derive(Debug)]
pub enum AppError {

    // Errores relacionados con el archivo de configuración:
    ConfigLoadError,
    FileDoesNotExist,
    FileInaccessible,

    //Errores de scanner
    RequestError,
    ResponseBodyError,
    HtmlParsingError,
}

// Implementa el trait `Display` para `AppError` para proporcionar descripciones.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::ConfigLoadError => write!(f, "Error al cargar el archivo de configuración."),
            AppError::FileDoesNotExist => write!(f, "El archivo config.json no existe."),
            AppError::FileInaccessible => write!(f, "No se puede acceder al archivo config.json. Verifica los permisos."),
            AppError::RequestError => write!(f, "Error de conexion"),
            AppError::ResponseBodyError => write!(f, "Error al obtener el body."),
            AppError::HtmlParsingError => write!(f, "Error al parsear el HTML."),
        }
    }
}

// Implementa el trait `Error` para `AppError`.
impl std::error::Error for AppError {}

