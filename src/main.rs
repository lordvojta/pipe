mod crypto;
mod pipe_server;
mod protocol;

use anyhow::{Context, Result};
use pipe_server::NamedPipeServer;
use protocol::{Request, Response};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const PIPE_NAME: &str = r"\\.\pipe\terminal_to_ps";
const KEY_FILE: &str = ".terminal_to_ps_key";

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
            // Load existing key
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
            // Generate new key
            let key = crypto::generate_key();
            let key_hex = hex::encode(key);

            fs::write(&key_path, key_hex)
                .context("Failed to write key file")?;

            println!("Generated new encryption key: {}", key_path.display());
            println!("IMPORTANT: Copy this key file to your PowerShell client location!");
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
                // Note: This sets the variable in the Rust process
                // PowerShell will need to handle setting it in its own session
                env::set_var(&name, &value);
                Response::success(Some(format!("Environment variable '{}' set", name)))
            }
            Request::SendData { key, data } => {
                println!("Received data - Key: {}, Data length: {} bytes", key, data.len());
                Response::success(Some(format!("Data received for key '{}'", key)))
            }
            Request::Ping => Response::Pong,
        }
    }

    fn process_encrypted_request(&self, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
        // Decrypt the request
        let request_json = crypto::decrypt(&encrypted_data, &self.key)
            .context("Failed to decrypt request")?;

        let request_str = String::from_utf8(request_json)
            .context("Invalid UTF-8 in request")?;

        let request: Request = serde_json::from_str(&request_str)
            .context("Failed to parse request JSON")?;

        println!("Request: {:?}", request);

        // Handle the request
        let response = self.handle_request(request);

        println!("Response: {:?}", response);

        // Serialize and encrypt response
        let response_json = serde_json::to_string(&response)
            .context("Failed to serialize response")?;

        let encrypted_response = crypto::encrypt(response_json.as_bytes(), &self.key)
            .context("Failed to encrypt response")?;

        Ok(encrypted_response)
    }

    fn run(&self) -> Result<()> {
        let pipe_server = NamedPipeServer::new(PIPE_NAME);

        println!("\n=== Terminal to PowerShell Bridge ===");
        println!("Pipe: {}", PIPE_NAME);
        println!("Ready to accept connections...\n");

        pipe_server.listen(|data| self.process_encrypted_request(data))
    }
}

fn main() -> Result<()> {
    let server = Server::new()?;
    server.run()
}
