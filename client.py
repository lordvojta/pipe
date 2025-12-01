#!/usr/bin/env python3
"""Simple client to send text to the terminal-to-ps server."""

import socket
import struct
import json
import os
import sys
from pathlib import Path

# Crypto imports
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305

def get_key():
    """Load the encryption key from ~/.terminal_to_ps_key"""
    key_path = Path.home() / ".terminal_to_ps_key"
    if not key_path.exists():
        print(f"Error: Key file not found at {key_path}")
        print("Run the server first to generate a key.")
        sys.exit(1)
    return bytes.fromhex(key_path.read_text().strip())

def encrypt(data: bytes, key: bytes) -> bytes:
    """Encrypt data using ChaCha20-Poly1305"""
    nonce = os.urandom(12)
    cipher = ChaCha20Poly1305(key)
    ciphertext = cipher.encrypt(nonce, data, None)
    return nonce + ciphertext

def decrypt(data: bytes, key: bytes) -> bytes:
    """Decrypt data using ChaCha20-Poly1305"""
    nonce = data[:12]
    ciphertext = data[12:]
    cipher = ChaCha20Poly1305(key)
    return cipher.decrypt(nonce, ciphertext, None)

def send_message(text: str):
    """Send a text message to the server"""
    key = get_key()

    # Create request
    request = {"type": "send_data", "key": "text", "data": text}
    request_json = json.dumps(request).encode()
    encrypted = encrypt(request_json, key)

    if sys.platform == "win32":
        # Windows named pipe
        import win32file
        pipe_name = r"\\.\pipe\terminal_to_ps"
        handle = win32file.CreateFile(
            pipe_name,
            win32file.GENERIC_READ | win32file.GENERIC_WRITE,
            0, None,
            win32file.OPEN_EXISTING,
            0, None
        )
        win32file.WriteFile(handle, encrypted)
        _, response = win32file.ReadFile(handle, 4096)
        win32file.CloseHandle(handle)
    else:
        # Unix domain socket
        socket_path = Path.home() / ".terminal_to_ps.sock"
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(str(socket_path))

        # Send length-prefixed message
        sock.sendall(struct.pack(">I", len(encrypted)))
        sock.sendall(encrypted)

        # Read response
        resp_len = struct.unpack(">I", sock.recv(4))[0]
        response = sock.recv(resp_len)
        sock.close()

    # Decrypt and display response
    decrypted = decrypt(response, key)
    result = json.loads(decrypted)
    print(f"Response: {json.dumps(result, indent=2)}")

def ping():
    """Send a ping to check if server is alive"""
    key = get_key()
    request = {"type": "ping"}
    request_json = json.dumps(request).encode()
    encrypted = encrypt(request_json, key)

    if sys.platform == "win32":
        import win32file
        pipe_name = r"\\.\pipe\terminal_to_ps"
        handle = win32file.CreateFile(
            pipe_name,
            win32file.GENERIC_READ | win32file.GENERIC_WRITE,
            0, None,
            win32file.OPEN_EXISTING,
            0, None
        )
        win32file.WriteFile(handle, encrypted)
        _, response = win32file.ReadFile(handle, 4096)
        win32file.CloseHandle(handle)
    else:
        socket_path = Path.home() / ".terminal_to_ps.sock"
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(str(socket_path))
        sock.sendall(struct.pack(">I", len(encrypted)))
        sock.sendall(encrypted)
        resp_len = struct.unpack(">I", sock.recv(4))[0]
        response = sock.recv(resp_len)
        sock.close()

    decrypted = decrypt(response, key)
    result = json.loads(decrypted)
    print(f"Server responded: {result['type']}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python client.py ping          - Check server")
        print("  python client.py send <text>   - Send text")
        sys.exit(1)

    cmd = sys.argv[1]
    if cmd == "ping":
        ping()
    elif cmd == "send" and len(sys.argv) > 2:
        text = " ".join(sys.argv[2:])
        send_message(text)
    else:
        print("Unknown command. Use 'ping' or 'send <text>'")
