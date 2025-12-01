#[cfg(windows)]
use anyhow::{Context, Result};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile, PIPE_ACCESS_DUPLEX};
#[cfg(windows)]
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT};

#[cfg(windows)]
const BUFFER_SIZE: u32 = 4096;

#[cfg(windows)]
pub struct NamedPipeServer {
    pipe_name: String,
}

#[cfg(windows)]
impl NamedPipeServer {
    pub fn new(pipe_name: impl Into<String>) -> Self {
        Self {
            pipe_name: pipe_name.into(),
        }
    }

    pub fn listen<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Vec<u8>) -> Result<Vec<u8>>,
    {
        println!("Starting named pipe server: {}", self.pipe_name);

        loop {
            let pipe_handle = self.create_pipe()?;
            println!("Waiting for client connection...");

            // Wait for a client to connect
            unsafe {
                ConnectNamedPipe(pipe_handle, None)
                    .context("Failed to connect to client")?;
            }

            println!("Client connected!");

            // Handle the connection
            match self.handle_connection(pipe_handle, &mut handler) {
                Ok(_) => println!("Connection handled successfully"),
                Err(e) => eprintln!("Error handling connection: {}", e),
            }

            // Close the pipe handle
            unsafe {
                let _ = CloseHandle(pipe_handle);
            }
        }
    }

    fn create_pipe(&self) -> Result<HANDLE> {
        let pipe_name_wide: Vec<u16> = self
            .pipe_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let pipe_handle = unsafe {
            CreateNamedPipeW(
                PCWSTR(pipe_name_wide.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                BUFFER_SIZE,
                BUFFER_SIZE,
                0,
                None,
            )
        };

        if pipe_handle == INVALID_HANDLE_VALUE {
            anyhow::bail!("Failed to create named pipe: {}", std::io::Error::last_os_error());
        }

        Ok(pipe_handle)
    }

    fn handle_connection<F>(&self, pipe_handle: HANDLE, handler: &mut F) -> Result<()>
    where
        F: FnMut(Vec<u8>) -> Result<Vec<u8>>,
    {
        // Read from pipe
        let mut buffer = vec![0u8; BUFFER_SIZE as usize];
        let mut bytes_read = 0u32;

        unsafe {
            ReadFile(
                pipe_handle,
                Some(&mut buffer),
                Some(&mut bytes_read),
                None,
            )
            .context("Failed to read from pipe")?;
        }

        let request_data = buffer[..bytes_read as usize].to_vec();

        // Process the request
        let response_data = handler(request_data)?;

        // Write response
        let mut bytes_written = 0u32;
        unsafe {
            WriteFile(
                pipe_handle,
                Some(&response_data),
                Some(&mut bytes_written),
                None,
            )
            .context("Failed to write to pipe")?;
        }

        Ok(())
    }
}

// Unix implementation using Unix domain sockets
#[cfg(unix)]
use anyhow::{Context, Result};
#[cfg(unix)]
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
const BUFFER_SIZE: usize = 4096;

#[cfg(unix)]
pub struct NamedPipeServer {
    socket_path: String,
}

#[cfg(unix)]
impl NamedPipeServer {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub fn listen<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Vec<u8>) -> Result<Vec<u8>>,
    {
        println!("Starting Unix domain socket server: {}", self.socket_path);

        // Remove existing socket file if it exists
        let path = Path::new(&self.socket_path);
        if path.exists() {
            std::fs::remove_file(path)
                .context("Failed to remove existing socket file")?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .context("Failed to bind Unix socket")?;

        println!("Waiting for client connection...");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Client connected!");
                    match self.handle_connection(stream, &mut handler) {
                        Ok(_) => println!("Connection handled successfully"),
                        Err(e) => eprintln!("Error handling connection: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_connection<F>(&self, mut stream: UnixStream, handler: &mut F) -> Result<()>
    where
        F: FnMut(Vec<u8>) -> Result<Vec<u8>>,
    {
        // Read length prefix (4 bytes, big endian)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)
            .context("Failed to read message length")?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        if msg_len > BUFFER_SIZE * 16 {
            anyhow::bail!("Message too large: {} bytes", msg_len);
        }

        // Read the message
        let mut buffer = vec![0u8; msg_len];
        stream.read_exact(&mut buffer)
            .context("Failed to read from socket")?;

        // Process the request
        let response_data = handler(buffer)?;

        // Write length prefix
        let resp_len = response_data.len() as u32;
        stream.write_all(&resp_len.to_be_bytes())
            .context("Failed to write response length")?;

        // Write response
        stream.write_all(&response_data)
            .context("Failed to write to socket")?;

        Ok(())
    }
}

#[cfg(unix)]
impl Drop for NamedPipeServer {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
