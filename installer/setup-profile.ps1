# Adds the sne setup line to a PowerShell profile, once.
# The line checks that sne exists, so it stays harmless if sne is uninstalled later.
param([string]$ProfilePath = $PROFILE)

$line = 'if (Get-Command sne -ErrorAction SilentlyContinue) { sne env --use-on-cd | Out-String | Invoke-Expression }'

$dir = Split-Path -Parent $ProfilePath
if (-not (Test-Path -LiteralPath $dir)) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
}
if ((Test-Path -LiteralPath $ProfilePath) -and (Select-String -LiteralPath $ProfilePath -SimpleMatch 'sne env' -Quiet)) {
    exit 0
}
Add-Content -LiteralPath $ProfilePath -Value $line
