[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $ExePath -PathType Leaf)) {
    throw "Executable does not exist: $ExePath"
}

$file = Get-Item -LiteralPath $ExePath
if ($file.Length -le 0) {
    throw "Executable is empty: $ExePath"
}

& $file.FullName --install-status
if ($LASTEXITCODE -ne 0) {
    throw "Installer status command failed with exit code $LASTEXITCODE"
}

Write-Host "Windows executable and installer status entrypoint validated: $($file.FullName)"
