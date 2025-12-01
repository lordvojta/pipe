mod crypto;
mod protocol;

use anyhow::{Context, Result};
use protocol::{Request, Response};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

const KEY_FILE: &str = ".terminal_to_ps_key";
const DEFAULT_PORT: u16 = 9876;

struct Server {
    key: [u8; 32],
}

impl Server {
    fn new() -> Result<Self> {
        let key = Self::load_or_generate_key()?;
        Ok(Self { key })
    }

    fn load_or_generate_key() -> Result<[u8; 32]> {
        let key_path = Self::get_key_path()?;

        if key_path.exists() {
            let key_hex = fs::read_to_string(&key_path)
                .context("Failed to read key file")?;
            let key_bytes = hex::decode(key_hex.trim())
                .context("Failed to decode key")?;

            if key_bytes.len() != 32 {
                anyhow::bail!("Invalid key length");
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);

            println!("Loaded encryption key from: {}", key_path.display());
            Ok(key)
        } else {
            let key = crypto::generate_key();
            let key_hex = hex::encode(key);

            fs::write(&key_path, &key_hex)
                .context("Failed to write key file")?;

            println!("Generated new encryption key: {}", key_path.display());
            println!("\nIMPORTANT: Copy this key to the other device:");
            println!("  {}", key_hex);
            Ok(key)
        }
    }

    fn get_key_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        Ok(home_dir.join(KEY_FILE))
    }

    fn handle_request(&self, request: Request) -> Response {
        match request {
            Request::GetEnv { name } => {
                match env::var(&name) {
                    Ok(value) => Response::success(Some(value)),
                    Err(_) => Response::error(format!("Environment variable '{}' not found", name)),
                }
            }
            Request::GetAllEnv => {
                let vars: HashMap<String, String> = env::vars().collect();
                Response::env_vars(vars)
            }
            Request::SetEnv { name, value } => {
                env::set_var(&name, &value);
                Response::success(Some(format!("Environment variable '{}' set", name)))
            }
            Request::SendData { key, data } => {
                println!("\n>>> MESSAGE: {}", data);
                Response::success(Some(format!("Received: {}", data)))
            }
            Request::Ping => Response::Pong,
        }
    }

    fn process_encrypted_request(&self, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
        let request_json = crypto::decrypt(&encrypted_data, &self.key)
            .context("Failed to decrypt request")?;

        let request_str = String::from_utf8(request_json)
            .context("Invalid UTF-8 in request")?;

        let request: Request = serde_json::from_str(&request_str)
            .context("Failed to parse request JSON")?;

        let response = self.handle_request(request);

        let response_json = serde_json::to_string(&response)
            .context("Failed to serialize response")?;

        let encrypted_response = crypto::encrypt(response_json.as_bytes(), &self.key)
            .context("Failed to encrypt response")?;

        Ok(encrypted_response)
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let peer = stream.peer_addr().ok();

        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        // Read message
        let mut buffer = vec![0u8; msg_len];
        stream.read_exact(&mut buffer)?;

        // Process
        let response = self.process_encrypted_request(buffer)?;

        // Write response length + data
        stream.write_all(&(response.len() as u32).to_be_bytes())?;
        stream.write_all(&response)?;

        if let Some(addr) = peer {
            println!("Handled request from {}", addr);
        }

        Ok(())
    }

    fn run(&self, port: u16) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)
            .with_context(|| format!("Failed to bind to {}", addr))?;

        println!("\n=== Secure Messenger Server ===");
        println!("Listening on port {}", port);
        println!("Waiting for connections...\n");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.handle_connection(stream) {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse().unwrap_or(DEFAULT_PORT)
    } else {
        DEFAULT_PORT
    };

    let server = Server::new()?;
    server.run(port)
}
