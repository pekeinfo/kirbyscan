use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::time::Duration;
use std::sync::RwLock;

use crate::config::Proxy;

impl Proxy {
    pub fn to_socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.ip, self.port).to_socket_addrs().unwrap().next().unwrap()
    }

    pub fn is_alive(&self, timeout: Duration) -> bool {
        let addr = self.to_socket_addr();
        TcpStream::connect_timeout(&addr, timeout).is_ok()
    }
}

pub struct ProxyManager {
    all_proxies: Vec<Proxy>,
    active_proxies: RwLock<Vec<Proxy>>,
    current_proxy_index: RwLock<Option<usize>>,
}

impl ProxyManager {
    pub fn new(proxies: Vec<Proxy>, timeout: Duration) -> Self {
        let active_proxies: Vec<Proxy> = proxies.iter()
            .filter(|&p| p.is_alive(timeout))
            .cloned()
            .collect();
        
        let current_proxy_index = active_proxies.last().map(|proxy| {
            proxies.iter().position(|x| *x == *proxy).unwrap()
        });

        ProxyManager {
            all_proxies: proxies,
            active_proxies: RwLock::new(active_proxies),
            current_proxy_index: RwLock::new(current_proxy_index),
        }
    }

    pub fn get_current_proxy(&self) -> Option<Proxy> {
        let current_index = self.current_proxy_index.read().unwrap();
        if let Some(index) = *current_index {
            Some(self.active_proxies.read().unwrap()[index].clone())
        } else {
            None
        }
    }

    pub fn discard_current_proxy(&self) {
        if let Some(proxy_index) = self.current_proxy_index.write().unwrap().take() {
            let mut active_proxies_lock = self.active_proxies.write().unwrap();
            active_proxies_lock.remove(proxy_index);
            self.select_next_proxy();
        }
    }

    fn select_next_proxy(&self) {
        let active_proxies_lock = self.active_proxies.read().unwrap();
        let new_current_index = active_proxies_lock.last().map(|proxy| {
            self.all_proxies.iter().position(|x| *x == *proxy).unwrap()
        });
        *self.current_proxy_index.write().unwrap() = new_current_index;
    }
}
