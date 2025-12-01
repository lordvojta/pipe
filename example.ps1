# Example usage of Terminal-to-PS Bridge

# Import the client module
. .\client.ps1

Write-Host "=== Terminal to PowerShell Bridge - Examples ==="
Write-Host ""

# Test connection
Write-Host "1. Testing connection..."
Test-Connection
Write-Host ""

# Get a specific environment variable
Write-Host "2. Getting PATH environment variable..."
$path = Get-EnvVar -Name "PATH"
if ($path) {
    Write-Host "PATH = $($path.Substring(0, [Math]::Min(100, $path.Length)))..."
}
Write-Host ""

# Get all environment variables
Write-Host "3. Getting all environment variables..."
$allVars = Get-AllEnvVars
if ($allVars) {
    Write-Host "Retrieved $($allVars.Count) environment variables"
    Write-Host "Sample variables:"
    $allVars.GetEnumerator() | Select-Object -First 5 | ForEach-Object {
        Write-Host "  $($_.Key) = $($_.Value.Substring(0, [Math]::Min(50, $_.Value.Length)))..."
    }
}
Write-Host ""

# Set an environment variable
Write-Host "4. Setting a custom environment variable..."
Set-EnvVar -Name "MY_SECRET_KEY" -Value "super-secret-value-12345"
Write-Host ""

# Send custom data
Write-Host "5. Sending custom data..."
Send-Data -Key "api_token" -Data "my-api-token-abc123"
Write-Host ""

Write-Host "=== Examples completed ==="
