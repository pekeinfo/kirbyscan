use serde::{Deserialize, Serialize};
use std::fs;
use crate::errors::AppError;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub proxy: bool,
    pub hilos: u32,
    pub timeout: u64, // timeout en segundos
    pub proxies: Option<Vec<Proxy>>,
    pub user_agent: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Proxy {
   pub ip: String,
   pub port: u16,
}

impl Config {
    pub fn load_from_file() -> Result<Self, AppError> {
        let config_content = fs::read_to_string("config.json")
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => AppError::FileDoesNotExist,
                std::io::ErrorKind::PermissionDenied => AppError::FileInaccessible,
                _ => AppError::ConfigLoadError,
            })?;

        let mut config: Config = serde_json::from_str(&config_content)
            .map_err(|_| AppError::ConfigLoadError)?;
        
        if !config.proxy {
            config.proxies = None;
        }
        
        // Verificación del número de hilos
        let recommended_threads = match std::thread::available_parallelism() {
            Ok(n) => n.get() as u32,
            Err(e) => {
                eprintln!("Ocurrió un error al determinar el número recomendado de hilos: {}", e);
                u32::MAX // Usa un valor alto para no modificar la configuración
            },
        };

        if config.hilos < recommended_threads {
            println!("Hilos recomendados: {}", recommended_threads);
        } else if config.hilos > recommended_threads {
            println!("Demasiados hilos en la configuración. Usando {} hilos recomendados.", recommended_threads);
            config.hilos = recommended_threads;
        }

        Ok(config)
    }
}
