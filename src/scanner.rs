use std::net::IpAddr;
use reqwest::{Client, Proxy as ReqProxy, StatusCode};
use scraper::{Html, Selector};
use crate::config::Config;
use crate::errors::AppError;
use crate::proxy::ProxyManager;
use std::sync::Arc;

pub struct Scanner {
    base_ip: IpAddr,
    port: u16,
    uri: String,
    client: Client,
    proxy_manager: Option<Arc<ProxyManager>>,
}

impl Scanner {
    fn build_client(config: &Config, proxy_manager: &Option<Arc<ProxyManager>>) -> Client {
        let timeout_duration = std::time::Duration::from_secs(config.timeout);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, config.user_agent.parse().unwrap());

        let mut client_builder = Client::builder()
            .timeout(timeout_duration)
            .tcp_keepalive(timeout_duration)
            .default_headers(headers);

        if let Some(pm) = proxy_manager {
            if let Some(proxy) = pm.get_current_proxy() {
                let proxy_str = format!("socks5://{}", proxy.to_socket_addr());
                if let Ok(proxy) = ReqProxy::all(&proxy_str) {
                    client_builder = client_builder.proxy(proxy);
                }
            }
        }

        client_builder.build().expect("Failed to create HTTP client")
    }

    pub fn new(base_ip: IpAddr, port: u16, uri: &str, config: &Config, proxy_manager: Option<Arc<ProxyManager>>) -> Self {
        let client = Scanner::build_client(config, &proxy_manager);
    
        Scanner {
            base_ip,
            port,
            uri: uri.to_string(),
            client,
            proxy_manager,
        }
    }


    pub async fn scan(&mut self, config: &Config) -> Result<(StatusCode, Option<String>), AppError> {
        let url = format!("http://{}:{}{}", self.base_ip, self.port, self.uri);
        let max_attempts = 2;
    
        for _ in 0..max_attempts {
            let response = self.client.get(&url).send().await;
    
            match response {
                Ok(res) => {
                    let status = res.status();
                    let body = res.text().await.map_err(|_| AppError::ResponseBodyError)?;
                    let title = Scanner::extract_title(&body)?;
                    return Ok((status, title));
                }
                Err(err) => {
                    if let Some(_pm) = &self.proxy_manager {
                        self.client = Scanner::build_client(config, &self.proxy_manager);
                    }
    
                    if !err.is_timeout() {
                        break;
                    }
                }
            }
        }
    
        Err(AppError::RequestError)
    }


    fn extract_title(html_content: &str) -> Result<Option<String>, AppError> {
        let fragment = Html::parse_document(html_content);
        let selector = Selector::parse("title").map_err(|_| AppError::HtmlParsingError)?;

        let title = fragment.select(&selector).next().map(|e| e.inner_html());

        Ok(title)
    }
}
