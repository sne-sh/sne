# Installs sne on Windows without admin rights.
#
#   irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
#
# Optional settings, as environment variables set before running:
#   SNE_VERSION         release tag to install, for example 0.1.0-beta.5 (default: latest)
#   SNE_INSTALL_DIR     where sne.exe goes (default: %LOCALAPPDATA%\Programs\sne)
#   SNE_NO_MODIFY_PATH  set to 1 to leave PATH alone
#   SNE_NO_PROFILE      set to 1 to leave the PowerShell profile alone

# Whether new terminals of this PowerShell (powershell.exe or pwsh.exe) refuse to load a profile.
# A new process is asked, without the policy this process may have been started with.
function Test-ProfileBlocked([string]$shell) {
    $saved = $env:PSExecutionPolicyPreference
    $env:PSExecutionPolicyPreference = $null
    try {
        $policy = & $shell -NoProfile -NonInteractive -Command Get-ExecutionPolicy 2>$null
        return "$policy".Trim() -in 'Restricted', 'AllSigned'
    }
    catch { return $false }
    finally { $env:PSExecutionPolicyPreference = $saved }
}

function Install-Sne {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'   # the progress bar makes downloads much slower in PowerShell 5.1
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $repo = 'sne-sh/sne'
    $base = if ($env:SNE_DOWNLOAD_BASE) { $env:SNE_DOWNLOAD_BASE.TrimEnd('/') } else { "https://github.com/$repo/releases/download" }
    $installDir = if ($env:SNE_INSTALL_DIR) { $env:SNE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\sne' }

    # A 32-bit PowerShell on 64-bit Windows reports the real architecture in PROCESSOR_ARCHITEW6432.
    $arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    $target = switch ($arch) {
        'AMD64' { 'x86_64-pc-windows-msvc' }
        'ARM64' { 'aarch64-pc-windows-msvc' }
        'x86' { 'i686-pc-windows-msvc' }
        default { throw "sne has no build for this processor ($arch)." }
    }

    $tag = $env:SNE_VERSION
    if (-not $tag -or $tag -eq 'latest') {
        $tag = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest").tag_name
        if (-not $tag) { throw 'Could not find the latest sne release.' }
    }

    $asset = "sne-$tag-$target.zip"
    $temp = Join-Path ([IO.Path]::GetTempPath()) "sne-install-$PID"
    New-Item -ItemType Directory -Force -Path $temp | Out-Null
    try {
        Write-Host "Downloading sne $tag ($target)..."
        $zip = Join-Path $temp $asset
        Invoke-WebRequest -UseBasicParsing "$base/$tag/$asset" -OutFile $zip
        $sums = Join-Path $temp 'SHA256SUMS.txt'
        Invoke-WebRequest -UseBasicParsing "$base/$tag/SHA256SUMS.txt" -OutFile $sums

        $expected = Get-Content $sums |
            Where-Object { ($_ -split '\s+', 2)[1] -replace '^\*', '' -eq $asset } |
            ForEach-Object { ($_ -split '\s+')[0] } |
            Select-Object -First 1
        if (-not $expected) { throw "$asset is not listed in SHA256SUMS.txt." }
        $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash
        if ($actual -ne $expected) { throw "Checksum mismatch for $asset. Nothing was installed." }

        Expand-Archive -LiteralPath $zip -DestinationPath $temp -Force
        $exe = Get-ChildItem -Path $temp -Recurse -Filter 'sne.exe' | Select-Object -First 1
        if (-not $exe) { throw "sne.exe was not found inside $asset." }
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        Copy-Item -LiteralPath $exe.FullName -Destination (Join-Path $installDir 'sne.exe') -Force
    }
    finally {
        Remove-Item -Recurse -Force -LiteralPath $temp -ErrorAction SilentlyContinue
    }
    $sne = Join-Path $installDir 'sne.exe'
    Write-Host "Installed $sne"

    # Run it once before touching PATH or any profile: Windows can refuse to start a program.
    try {
        # Windows PowerShell turns anything a program writes to its error stream into an error.
        $ErrorActionPreference = 'Continue'
        & $sne --version 2>$null | Out-Null
        $ErrorActionPreference = 'Stop'
        if ($LASTEXITCODE -ne 0) { throw "it started, but stopped with exit code $LASTEXITCODE." }
    }
    catch {
        $ErrorActionPreference = 'Stop'
        Write-Host ''
        Write-Host "Windows did not run sne.exe: $($_.Exception.Message.Trim())"
        if ("$_" -match 'Application Control') {
            Write-Host ''
            Write-Host 'This is Smart App Control or an application policy of your organization. It blocks programs'
            Write-Host 'that are not signed, and sne is not signed yet. sne cannot work on this PC until one of these:'
            Write-Host '  - Smart App Control is turned off: Windows Security > App & browser control >'
            Write-Host '    Smart App Control settings. Windows does not let you turn it on again later.'
            Write-Host '  - Your administrator allows sne.exe, if the policy comes from your organization.'
        }
        Write-Host ''
        Write-Host "sne.exe was left in $installDir. Your PATH and PowerShell profile were not changed."
        return
    }

    if ($env:SNE_NO_MODIFY_PATH -ne '1') {
        # Two folders go on PATH: sne itself, and the link to the default Node version. With the
        # second one, node works in every terminal and program without any shell setup.
        $dataDir = if ($env:SNE_DIR) { $env:SNE_DIR } else { Join-Path $env:LOCALAPPDATA 'sne' }
        $pathDirs = @($installDir, (Join-Path $dataDir 'aliases\default'))
        # Read and write the raw registry value, so entries such as %USERPROFILE%\bin are kept as written.
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
        try {
            $changed = $false
            foreach ($dir in $pathDirs) {
                $userPath = [string]$key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
                if (($userPath -split ';') -notcontains $dir) {
                    $key.SetValue('Path', ($userPath.TrimEnd(';') + ';' + $dir).TrimStart(';'), 'ExpandString')
                    $changed = $true
                    Write-Host "Added $dir to your PATH"
                }
                if (($env:Path -split ';') -notcontains $dir) { $env:Path = "$dir;$env:Path" }
            }
            if ($changed) {
                # Writing any user variable through .NET tells running programs that the environment changed.
                [Environment]::SetEnvironmentVariable('SNE_INSTALL_NOTIFY', '1', 'User')
                [Environment]::SetEnvironmentVariable('SNE_INSTALL_NOTIFY', $null, 'User')
            }
        }
        finally {
            $key.Close()
        }
    }

    if ($env:SNE_NO_PROFILE -ne '1') {
        $line = 'if (Get-Command sne -ErrorAction SilentlyContinue) { sne env --use-on-cd | Out-String | Invoke-Expression }'
        # Windows PowerShell 5.1 and PowerShell 7 keep separate profiles. Set up the one running this
        # script, and the other one too when that edition is installed, so sne works in both.
        $documents = [Environment]::GetFolderPath('MyDocuments')
        $profiles = @($PROFILE)
        if ($documents) {
            $profiles += Join-Path $documents 'WindowsPowerShell\Microsoft.PowerShell_profile.ps1'
            if (Get-Command pwsh -ErrorAction SilentlyContinue) {
                $profiles += Join-Path $documents 'PowerShell\Microsoft.PowerShell_profile.ps1'
            }
        }
        foreach ($profilePath in ($profiles | Select-Object -Unique)) {
            # Where the execution policy blocks scripts, a profile only prints an error in every new
            # terminal. sne works without the line there; only per-terminal switching is missing.
            $shell = if ($profilePath -like '*\WindowsPowerShell\*') { 'powershell.exe' } else { 'pwsh.exe' }
            if (Test-ProfileBlocked $shell) {
                $name = if ($shell -eq 'pwsh.exe') { 'PowerShell 7' } else { 'Windows PowerShell' }
                Write-Host "$name does not allow profile scripts on this PC, so its profile was left alone."
                Write-Host '  node and sne work without it. To also switch versions per terminal, run'
                Write-Host '  Set-ExecutionPolicy -Scope CurrentUser RemoteSigned   and then this installer again.'
                continue
            }
            $profileDir = Split-Path -Parent $profilePath
            if (-not (Test-Path -LiteralPath $profileDir)) { New-Item -ItemType Directory -Force -Path $profileDir | Out-Null }
            if (-not ((Test-Path -LiteralPath $profilePath) -and (Select-String -LiteralPath $profilePath -SimpleMatch 'sne env' -Quiet))) {
                Add-Content -LiteralPath $profilePath -Value $line
                Write-Host "Added the sne setup line to $profilePath"
            }
        }
    }

    Write-Host ''
    Write-Host 'Done. Open a new terminal, then run:  sne install lts'
}

Install-Sne
