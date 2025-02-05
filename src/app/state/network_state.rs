use std::sync::mpsc;
use local_ip_address::local_ip;

pub struct NetworkState {
    pub address: String,
    pub auto_detect_ip: bool, // New flag for toggling auto/manual IP selection
    stop_tx: Option<mpsc::Sender<()>>,
    broadcast_stop_tx: Option<mpsc::Sender<()>>,
    record_stop_tx: Option<mpsc::Sender<()>>,
}

impl NetworkState {
    pub fn new() -> Self {
        let ip_address = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => String::from("Unable to get IP"),
        };

        Self {
            address: ip_address,
            auto_detect_ip: true, // Default to auto-detection
            stop_tx: None,
            broadcast_stop_tx: None,
            record_stop_tx: None,
        }
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn set_address(&mut self, value: String) {
        self.address = value;
    }

    pub fn get_broadcast_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.broadcast_stop_tx.clone()
    }

    pub fn set_broadcast_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.broadcast_stop_tx = tx;
    }

    pub fn get_record_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.record_stop_tx.clone()
    }

    pub fn set_record_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.record_stop_tx = tx;
    }

    pub fn get_stop_tx(&self) -> Option<mpsc::Sender<()>> {
        self.stop_tx.clone()
    }

    pub fn set_stop_tx(&mut self, tx: Option<mpsc::Sender<()>>) {
        self.stop_tx = tx;
    }
}
