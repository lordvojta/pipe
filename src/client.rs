mod crypto;
mod protocol;

use anyhow::{Context, Result};
use protocol::{Request, Response};
use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

const KEY_FILE: &str = ".terminal_to_ps_key";

#[cfg(windows)]
const PIPE_NAME: &str = r"\\.\pipe\terminal_to_ps";

#[cfg(unix)]
fn get_socket_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".terminal_to_ps.sock")
}

fn get_key_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home_dir.join(KEY_FILE))
}

fn load_key() -> Result<[u8; 32]> {
    let key_path = get_key_path()?;
    let key_hex = fs::read_to_string(&key_path)
        .with_context(|| format!("Key file not found: {}\nRun the server first.", key_path.display()))?;
    let key_bytes = hex::decode(key_hex.trim()).context("Failed to decode key")?;

    if key_bytes.len() != 32 {
        anyhow::bail!("Invalid key length");
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

fn send_request(request: &Request, key: &[u8; 32]) -> Result<Response> {
    let request_json = serde_json::to_string(request)?;
    let encrypted = crypto::encrypt(request_json.as_bytes(), key)?;

    let response_bytes = send_raw(&encrypted)?;

    let decrypted = crypto::decrypt(&response_bytes, key)?;
    let response_str = String::from_utf8(decrypted)?;
    let response: Response = serde_json::from_str(&response_str)?;

    Ok(response)
}

#[cfg(windows)]
fn send_raw(data: &[u8]) -> Result<Vec<u8>> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Foundation::GENERIC_WRITE;
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE, OPEN_EXISTING,
    };

    let pipe_name_wide: Vec<u16> = PIPE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(pipe_name_wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };

    // Write
    let mut bytes_written = 0u32;
    unsafe {
        WriteFile(handle, Some(data), Some(&mut bytes_written), None)
            .context("Failed to write to pipe")?;
    }

    // Read response
    let mut buffer = vec![0u8; 4096];
    let mut bytes_read = 0u32;
    unsafe {
        ReadFile(handle, Some(&mut buffer), Some(&mut bytes_read), None)
            .context("Failed to read from pipe")?;
    }

    unsafe {
        windows::Win32::Foundation::CloseHandle(handle)?;
    }

    Ok(buffer[..bytes_read as usize].to_vec())
}

#[cfg(unix)]
fn send_raw(data: &[u8]) -> Result<Vec<u8>> {
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path)
        .with_context(|| format!("Cannot connect to server at {}", socket_path.display()))?;

    // Send length prefix + data
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(data)?;

    // Read response length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let resp_len = u32::from_be_bytes(len_buf) as usize;

    // Read response
    let mut buffer = vec![0u8; resp_len];
    stream.read_exact(&mut buffer)?;

    Ok(buffer)
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
    eprintln!("Usage:");
    eprintln!("  client ping                  - Check server connection");
    eprintln!("  client send <text>           - Send text data");
    eprintln!("  client get <VAR>             - Get environment variable");
    eprintln!("  client getall                - Get all environment variables");
    eprintln!("  client set <VAR> <VALUE>     - Set environment variable");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let key = load_key()?;

    let request = match args[1].as_str() {
        "ping" => Request::Ping,
        "send" => {
            if args.len() < 3 {
                eprintln!("Usage: client send <text>");
                std::process::exit(1);
            }
            let text = args[2..].join(" ");
            Request::SendData {
                key: "text".to_string(),
                data: text,
            }
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("Usage: client get <VAR>");
                std::process::exit(1);
            }
            Request::GetEnv { name: args[2].clone() }
        }
        "getall" => Request::GetAllEnv,
        "set" => {
            if args.len() < 4 {
                eprintln!("Usage: client set <VAR> <VALUE>");
                std::process::exit(1);
            }
            Request::SetEnv {
                name: args[2].clone(),
                value: args[3..].join(" "),
            }
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    let response = send_request(&request, &key)?;
    print_response(&response);

    Ok(())
}
