extern crate reqwest;
extern crate scraper;

use std::net::IpAddr;

mod config;
mod proxy;
mod scanner;
mod errors;
use std::time::Duration;

use crate::proxy::ProxyManager;
use config::Config;
use scanner::Scanner;

use ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;
use clap::App;
use clap::Arg;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use tokio::runtime::Runtime;
use std::sync::Arc;

fn generate_ips_in_subnet(cidr: &str) -> Result<Vec<Ipv4Addr>, ipnetwork::IpNetworkError> {
    let network: Ipv4Network = cidr.parse()?;
    Ok(network.iter().collect())
}

fn main() {
    let matches = App::new("KirbyScan")
        .arg(Arg::new("IP")
            .required(true)
            .index(1)
            .help("IP o rango IP para escanear (e.g., 192.168.1.1 o 192.168.1.1/24)"))
        .arg(Arg::new("PORT")
            .index(2)
            .default_value("80")
            .help("Puerto para escanear. Por defecto es 80."))
        .arg(Arg::new("URI")
            .index(3)
            .default_value("/")
            .help("URI para el escaneo. Por defecto es '/'."))
        .get_matches();

    let ip_or_cidr = matches.value_of("IP").unwrap();
    let mut cidr = String::from(ip_or_cidr);

    if !ip_or_cidr.contains('/') {
        cidr.push_str("/24");
    }

    let ips = match generate_ips_in_subnet(&cidr) {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Error al analizar la notación CIDR: {:?}", e);
            return;
        }
    };

    let port = matches.value_of("PORT").unwrap();
    
    let port_num: u16 = match port.parse() {
        Ok(p) if (1..=65535).contains(&p) => p,
        _ => {
            eprintln!("Error: invalid port number '{}'", port);
            return;
        }
    };

    let uri = matches.value_of("URI").unwrap();
    let config = Config::load_from_file().unwrap_or_else(|e| {
        println!("Error cargando configuración: {}", e);
        std::process::exit(1);
    });

  
    let proxy_manager = if let Some(ref proxies) = config.proxies {
        Some(Arc::new(ProxyManager::new(proxies.to_vec(), Duration::from_secs(config.timeout))))
    } else {
        None
    }; 

    let pool = ThreadPoolBuilder::new()
        .num_threads(config.hilos as usize)
        .build()
        .unwrap();

    pool.install(|| {
        // Suponiendo que tienes una función `get_ips_from_cidr` que devuelve un Vec<Ipv4Addr>
       
        let rt = Runtime::new().unwrap();

        ips.par_iter().for_each(|&ip| {
            rt.block_on(async {
                // Crea una nueva instancia de Scanner para la IP actual
                let mut scanner = Scanner::new(IpAddr::V4(ip), port_num, &uri, &config, proxy_manager.clone());
                
                match scanner.scan(&config).await {
                    Ok((status, title)) => {
                        println!("{}:{}{} - Status: {:?} - Title: {:?}", ip, port, uri, status, title.unwrap_or_else(|| "No title".to_string()));
                    },
                    Err(e) => {
                        //print!("{}",e);
                    }
                }
            });
        });
    });

}
