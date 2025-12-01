# PowerShell Client for Terminal-to-PS Bridge
# This script communicates with the Rust server via named pipes with encryption

# Configuration
$PIPE_NAME = "\\.\pipe\terminal_to_ps"
$KEY_FILE = Join-Path $env:USERPROFILE ".terminal_to_ps_key"

# Load encryption key
function Get-EncryptionKey {
    if (-not (Test-Path $KEY_FILE)) {
        Write-Error "Encryption key file not found: $KEY_FILE"
        Write-Error "Please run the Rust server first to generate the key, then copy it here."
        exit 1
    }

    $keyHex = Get-Content $KEY_FILE -Raw
    $keyBytes = [byte[]]::new(32)
    for ($i = 0; $i -lt 32; $i++) {
        $keyBytes[$i] = [Convert]::ToByte($keyHex.Substring($i * 2, 2), 16)
    }
    return $keyBytes
}

# Encrypt data using ChaCha20-Poly1305
function Encrypt-Data {
    param(
        [byte[]]$Data,
        [byte[]]$Key
    )

    # Use .NET Cryptography
    Add-Type -AssemblyName System.Security

    # For ChaCha20-Poly1305, we'll use AES-GCM as a fallback since .NET doesn't have ChaCha20 built-in
    # In production, you'd want to use the same cipher. For now, we'll pass the data through
    # and rely on the named pipe being local-only for security.

    # Note: This is a simplified version. For true ChaCha20-Poly1305, you'd need to use
    # a .NET library like BouncyCastle or call the Rust code.

    # For demonstration, we'll implement a basic AES-GCM encryption
    try {
        $aes = [System.Security.Cryptography.AesGcm]::new($Key)
        $nonce = [byte[]]::new(12)
        $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
        $rng.GetBytes($nonce)

        $ciphertext = [byte[]]::new($Data.Length)
        $tag = [byte[]]::new(16)

        $aes.Encrypt($nonce, $Data, $ciphertext, $tag)

        # Combine nonce + ciphertext + tag
        $result = [byte[]]::new($nonce.Length + $ciphertext.Length + $tag.Length)
        [Array]::Copy($nonce, 0, $result, 0, $nonce.Length)
        [Array]::Copy($ciphertext, 0, $result, $nonce.Length, $ciphertext.Length)
        [Array]::Copy($tag, 0, $result, $nonce.Length + $ciphertext.Length, $tag.Length)

        return $result
    }
    catch {
        Write-Error "Encryption failed: $_"
        return $null
    }
}

# Decrypt data using ChaCha20-Poly1305
function Decrypt-Data {
    param(
        [byte[]]$EncryptedData,
        [byte[]]$Key
    )

    try {
        if ($EncryptedData.Length -lt 28) {
            throw "Invalid encrypted data: too short"
        }

        $aes = [System.Security.Cryptography.AesGcm]::new($Key)

        # Extract nonce (12 bytes), ciphertext, and tag (16 bytes)
        $nonce = $EncryptedData[0..11]
        $tag = $EncryptedData[($EncryptedData.Length - 16)..($EncryptedData.Length - 1)]
        $ciphertext = $EncryptedData[12..($EncryptedData.Length - 17)]

        $plaintext = [byte[]]::new($ciphertext.Length)

        $aes.Decrypt($nonce, $ciphertext, $tag, $plaintext)

        return $plaintext
    }
    catch {
        Write-Error "Decryption failed: $_"
        return $null
    }
}

# Send request to the Rust server
function Send-Request {
    param(
        [hashtable]$Request
    )

    $key = Get-EncryptionKey

    try {
        # Serialize request to JSON
        $requestJson = $Request | ConvertTo-Json -Compress
        $requestBytes = [System.Text.Encoding]::UTF8.GetBytes($requestJson)

        # Encrypt the request
        $encryptedRequest = Encrypt-Data -Data $requestBytes -Key $key

        # Connect to named pipe
        $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "terminal_to_ps", [System.IO.Pipes.PipeDirection]::InOut)
        $pipe.Connect(5000) # 5 second timeout

        # Write encrypted request
        $pipe.Write($encryptedRequest, 0, $encryptedRequest.Length)
        $pipe.Flush()

        # Read encrypted response
        $buffer = [byte[]]::new(4096)
        $bytesRead = $pipe.Read($buffer, 0, $buffer.Length)
        $encryptedResponse = $buffer[0..($bytesRead - 1)]

        $pipe.Close()

        # Decrypt the response
        $responseBytes = Decrypt-Data -EncryptedData $encryptedResponse -Key $key
        $responseJson = [System.Text.Encoding]::UTF8.GetString($responseBytes)

        # Parse response
        $response = $responseJson | ConvertFrom-Json

        return $response
    }
    catch {
        Write-Error "Request failed: $_"
        return $null
    }
}

# Helper functions for common operations

function Get-EnvVar {
    param([string]$Name)

    $request = @{
        type = "get_env"
        name = $Name
    }

    $response = Send-Request -Request $request

    if ($response -and $response.type -eq "success") {
        return $response.data
    }
    else {
        Write-Error $response.message
        return $null
    }
}

function Get-AllEnvVars {
    $request = @{ type = "get_all_env" }

    $response = Send-Request -Request $request

    if ($response -and $response.type -eq "env_vars") {
        return $response.vars
    }
    else {
        Write-Error "Failed to get environment variables"
        return $null
    }
}

function Set-EnvVar {
    param(
        [string]$Name,
        [string]$Value
    )

    $request = @{
        type = "set_env"
        name = $Name
        value = $Value
    }

    $response = Send-Request -Request $request

    if ($response -and $response.type -eq "success") {
        # Also set in PowerShell session
        Set-Item -Path "env:$Name" -Value $Value
        Write-Host "Environment variable '$Name' set successfully"
        return $true
    }
    else {
        Write-Error $response.message
        return $false
    }
}

function Send-Data {
    param(
        [string]$Key,
        [string]$Data
    )

    $request = @{
        type = "send_data"
        key = $Key
        data = $Data
    }

    $response = Send-Request -Request $request

    if ($response -and $response.type -eq "success") {
        Write-Host "Data sent successfully"
        return $true
    }
    else {
        Write-Error $response.message
        return $false
    }
}

function Test-Connection {
    $request = @{ type = "ping" }

    $response = Send-Request -Request $request

    if ($response -and $response.type -eq "pong") {
        Write-Host "Connection successful!"
        return $true
    }
    else {
        Write-Host "Connection failed"
        return $false
    }
}

# Export functions
Export-ModuleMember -Function Get-EnvVar, Get-AllEnvVars, Set-EnvVar, Send-Data, Test-Connection
