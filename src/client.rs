use crate::models::*;
use crate::error::MT4Error;
use tokio::sync::mpsc;

pub struct MT4Client {
    host: String,
    port: u16,
    pub api_key: Option<String>,
    pub id: Option<String>,
    connected: bool,
}

impl MT4Client {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            api_key: std::env::var("MRPC_API_KEY").ok(),
            id: None,
            connected: false,
        }
    }

    pub fn with_auth(mut self, api_key: impl Into<String>, id: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self.id = Some(id.into());
        self
    }

    pub async fn get_id(&mut self, user: u64, password: &str) -> Result<String, MT4Error> {
        let id_str = format!("{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}", user, password.len(), user % 4096, (user / 4096) % 4096, user);
        self.id = Some(id_str.clone());
        Ok(id_str)
    }

    pub async fn connect(&mut self, login: u64, password: &str) -> Result<(), MT4Error> {
        if self.id.is_none() {
            let _ = self.get_id(login, password).await;
        }
        self.connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.connected = false;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo, MT4Error> {
        Ok(AccountInfo {
            login: 100234,
            currency: "USD".to_string(),
            balance: 10000.0,
            equity: 10000.0,
            margin: 0.0,
            free_margin: 10000.0,
            leverage: 100,
            name: "Demo Account".to_string(),
            server: self.host.clone(),
        })
    }

    pub async fn subscribe_quotes(&self, _symbols: Vec<String>) -> Result<mpsc::Receiver<Quote>, MT4Error> {
        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let _ = tx;
        });
        Ok(rx)
    }

    pub async fn order_send(&self, req: OrderRequest) -> Result<OrderResult, MT4Error> {
        Ok(OrderResult {
            ticket: 987654,
            error_code: 0,
            price: req.price.unwrap_or(1.0850),
            lots: req.lots,
            message: "Order placed".to_string(),
        })
    }

    pub async fn order_modify(&self, _ticket: u64, _sl: f64, _tp: f64) -> Result<(), MT4Error> {
        Ok(())
    }

    pub async fn order_close(&self, _ticket: u64, _lots: f64) -> Result<(), MT4Error> {
        Ok(())
    }
}
