mod crypto;
mod protocol;

use anyhow::{Context, Result};
use protocol::{Request, Response};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;

const KEY_FILE: &str = ".terminal_to_ps_key";

fn get_key_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home_dir.join(KEY_FILE))
}

fn load_key() -> Result<[u8; 32]> {
    let key_path = get_key_path()?;
    let key_hex = fs::read_to_string(&key_path)
        .with_context(|| format!("Key file not found: {}\nRun the server first or copy the key.", key_path.display()))?;
    let key_bytes = hex::decode(key_hex.trim()).context("Failed to decode key")?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length");
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

fn send_request(addr: &str, request: &Request, key: &[u8; 32]) -> Result<Response> {
    let request_json = serde_json::to_string(request)?;
    let encrypted = crypto::encrypt(request_json.as_bytes(), key)?;

    let mut stream = TcpStream::connect(addr)
        .with_context(|| format!("Cannot connect to {}", addr))?;

    // Send length + data
    stream.write_all(&(encrypted.len() as u32).to_be_bytes())?;
    stream.write_all(&encrypted)?;

    // Read response length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let resp_len = u32::from_be_bytes(len_buf) as usize;

    // Read response
    let mut buffer = vec![0u8; resp_len];
    stream.read_exact(&mut buffer)?;

    let decrypted = crypto::decrypt(&buffer, key)?;
    let response_str = String::from_utf8(decrypted)?;
    let response: Response = serde_json::from_str(&response_str)?;

    Ok(response)
}

fn print_response(response: &Response) {
    match response {
        Response::Success { data } => {
            if let Some(d) = data {
                println!("{}", d);
            } else {
                println!("OK");
            }
        }
        Response::EnvVars { vars } => {
            for (k, v) in vars {
                println!("{}={}", k, v);
            }
        }
        Response::Error { message } => {
            eprintln!("Error: {}", message);
        }
        Response::Pong => {
            println!("pong");
        }
    }
}

fn print_usage() {
    eprintln!("Usage: client <host:port> <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  ping                  - Test connection");
    eprintln!("  send <message>        - Send a message");
    eprintln!("  get <VAR>             - Get environment variable");
    eprintln!("  set <VAR> <VALUE>     - Set environment variable");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  client 192.168.1.100:9876 send \"Hello!\"");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let addr = &args[1];
    let cmd = &args[2];

    let key = load_key()?;

    let request = match cmd.as_str() {
        "ping" => Request::Ping,
        "send" => {
            if args.len() < 4 {
                eprintln!("Usage: client <host:port> send <message>");
                std::process::exit(1);
            }
            let text = args[3..].join(" ");
            Request::SendData {
                key: "text".to_string(),
                data: text,
            }
        }
        "get" => {
            if args.len() < 4 {
                eprintln!("Usage: client <host:port> get <VAR>");
                std::process::exit(1);
            }
            Request::GetEnv { name: args[3].clone() }
        }
        "set" => {
            if args.len() < 5 {
                eprintln!("Usage: client <host:port> set <VAR> <VALUE>");
                std::process::exit(1);
            }
            Request::SetEnv {
                name: args[3].clone(),
                value: args[4..].join(" "),
            }
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    let response = send_request(addr, &request, &key)?;
    print_response(&response);

    Ok(())
}
