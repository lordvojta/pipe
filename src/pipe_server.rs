#[cfg(windows)]
use anyhow::{Context, Result};
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
#[cfg(windows)]
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_ACCESS_DUPLEX, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT};

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

        if let Err(e) = pipe_handle {
            anyhow::bail!("Failed to create named pipe: {}", e);
        }

        Ok(pipe_handle.unwrap())
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
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
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
                Some(response_data.as_ptr() as *const _),
                response_data.len() as u32,
                Some(&mut bytes_written),
                None,
            )
            .context("Failed to write to pipe")?;
        }

        Ok(())
    }
}

#[cfg(not(windows))]
pub struct NamedPipeServer;

#[cfg(not(windows))]
impl NamedPipeServer {
    pub fn new(_pipe_name: impl Into<String>) -> Self {
        Self
    }

    pub fn listen<F>(&self, _handler: F) -> anyhow::Result<()>
    where
        F: FnMut(Vec<u8>) -> anyhow::Result<Vec<u8>>,
    {
        anyhow::bail!("Named pipes are only supported on Windows")
    }
}
