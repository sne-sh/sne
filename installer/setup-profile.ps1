# Adds the sne setup line to a PowerShell profile, once.
# The line checks that sne exists, so it stays harmless if sne is uninstalled later.
param([string]$ProfilePath = $PROFILE)

$line = 'if (Get-Command sne -ErrorAction SilentlyContinue) { sne env --use-on-cd | Out-String | Invoke-Expression }'

# Where the execution policy blocks scripts, a profile only prints an error in every new terminal.
# sne works without the line there, so leave the profile alone. The setup wizard starts this
# script with its own policy, so a new process is asked what new terminals will use.
$saved = $env:PSExecutionPolicyPreference
$env:PSExecutionPolicyPreference = $null
try {
    $policy = & (Get-Process -Id $PID).Path -NoProfile -NonInteractive -Command Get-ExecutionPolicy 2>$null
    if ("$policy".Trim() -in 'Restricted', 'AllSigned') { exit 0 }
}
catch {}
finally { $env:PSExecutionPolicyPreference = $saved }

$dir = Split-Path -Parent $ProfilePath
if (-not (Test-Path -LiteralPath $dir)) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
}
if ((Test-Path -LiteralPath $ProfilePath) -and (Select-String -LiteralPath $ProfilePath -SimpleMatch 'sne env' -Quiet)) {
    exit 0
}
Add-Content -LiteralPath $ProfilePath -Value $line
