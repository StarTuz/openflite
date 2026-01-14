use crate::SimClient;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

pub struct XPlaneClient {
    socket: Option<UdpSocket>,
    address: String,
    cache: Arc<Mutex<HashMap<String, f64>>>,
    subscriptions: HashMap<String, i32>,
}

impl XPlaneClient {
    pub fn new(address: &str) -> Self {
        Self {
            socket: None,
            address: address.to_string(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, variable: &str, frequency: i32) -> Result<()> {
        if let Some(socket) = &self.socket {
            let index = self.subscriptions.len() as i32 + 1;
            self.subscriptions.insert(variable.to_string(), index);

            let mut buf = [0u8; 413];
            buf[0..4].copy_from_slice(b"RREF");
            buf[4] = 0;
            buf[5..9].copy_from_slice(&frequency.to_le_bytes());
            buf[9..13].copy_from_slice(&index.to_le_bytes());

            let path_bytes = variable.as_bytes();
            let len = path_bytes.len().min(400);
            buf[13..13 + len].copy_from_slice(&path_bytes[..len]);

            socket.send_to(&buf[..13 + len + 1], &self.address)?;
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }
}

impl SimClient for XPlaneClient {
    fn connect(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        self.socket = Some(socket);
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.socket = None;
        Ok(())
    }

    fn read_variable(&mut self, variable: &str) -> Result<f64> {
        let cache = self.cache.lock().unwrap();
        cache
            .get(variable)
            .copied()
            .ok_or_else(|| anyhow!("Variable {} not found or not yet received", variable))
    }

    fn write_variable(&mut self, variable: &str, value: f64) -> Result<()> {
        if let Some(socket) = &self.socket {
            let mut buf = [0u8; 509];
            buf[0..4].copy_from_slice(b"DREF");
            buf[4] = 0;

            let value_bytes = (value as f32).to_le_bytes();
            buf[5..9].copy_from_slice(&value_bytes);

            let path_bytes = variable.as_bytes();
            let len = path_bytes.len().min(500);
            buf[9..9 + len].copy_from_slice(&path_bytes[..len]);

            socket.send_to(&buf[..9 + len + 1], &self.address)?;
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    fn execute_command(&mut self, command: &str) -> Result<()> {
        if let Some(socket) = &self.socket {
            let mut buf = [0u8; 505];
            buf[0..4].copy_from_slice(b"CMND");
            buf[4] = 0;

            let path_bytes = command.as_bytes();
            let len = path_bytes.len().min(500);
            buf[5..5 + len].copy_from_slice(&path_bytes[..len]);

            socket.send_to(&buf[..5 + len + 1], &self.address)?;
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    fn poll(&mut self) -> Result<()> {
        if let Some(socket) = &self.socket {
            let mut buf = [0u8; 4096];
            while let Ok((amt, _)) = socket.recv_from(&mut buf) {
                if amt >= 5 && &buf[0..4] == b"RREF" {
                    // X-Plane sends RREF packets with:
                    // 5 bytes header (RREF + 0)
                    // then multiple 8-byte entries: 4 bytes index, 4 bytes value
                    let mut pos = 5;
                    while pos + 8 <= amt {
                        let mut index_bytes = [0u8; 4];
                        index_bytes.copy_from_slice(&buf[pos..pos + 4]);
                        let index = i32::from_le_bytes(index_bytes);

                        let mut val_bytes = [0u8; 4];
                        val_bytes.copy_from_slice(&buf[pos + 4..pos + 8]);
                        let val = f32::from_le_bytes(val_bytes);

                        // Map index back to name
                        if let Some(name) = self
                            .subscriptions
                            .iter()
                            .find(|(_, &v)| v == index)
                            .map(|(k, _)| k.clone())
                        {
                            let mut cache = self.cache.lock().unwrap();
                            cache.insert(name, val as f64);
                        }
                        pos += 8;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_all_variables(&self) -> HashMap<String, f64> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }
}
