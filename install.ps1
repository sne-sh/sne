# Installs sne on Windows without admin rights.
#
#   irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex
#
# Optional settings, as environment variables set before running:
#   SNE_VERSION         release tag to install, for example 0.1.0-beta.1 (default: latest)
#   SNE_INSTALL_DIR     where sne.exe goes (default: %LOCALAPPDATA%\Programs\sne)
#   SNE_NO_MODIFY_PATH  set to 1 to leave PATH alone
#   SNE_NO_PROFILE      set to 1 to leave the PowerShell profile alone
#   SNE_NOTIFY          set to 1 to get a desktop notification when a new sne version is out

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
    Write-Host "Installed $(Join-Path $installDir 'sne.exe')"

    if ($env:SNE_NO_MODIFY_PATH -ne '1') {
        # Read and write the raw registry value, so entries such as %USERPROFILE%\bin are kept as written.
        $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
        try {
            $userPath = [string]$key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
            if (($userPath -split ';') -notcontains $installDir) {
                $key.SetValue('Path', ($userPath.TrimEnd(';') + ';' + $installDir).TrimStart(';'), 'ExpandString')
                # Writing any user variable through .NET tells running programs that the environment changed.
                [Environment]::SetEnvironmentVariable('SNE_INSTALL_NOTIFY', '1', 'User')
                [Environment]::SetEnvironmentVariable('SNE_INSTALL_NOTIFY', $null, 'User')
                Write-Host "Added $installDir to your PATH"
            }
        }
        finally {
            $key.Close()
        }
        if (($env:Path -split ';') -notcontains $installDir) { $env:Path = "$installDir;$env:Path" }
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
            $profileDir = Split-Path -Parent $profilePath
            if (-not (Test-Path -LiteralPath $profileDir)) { New-Item -ItemType Directory -Force -Path $profileDir | Out-Null }
            if (-not ((Test-Path -LiteralPath $profilePath) -and (Select-String -LiteralPath $profilePath -SimpleMatch 'sne env' -Quiet))) {
                Add-Content -LiteralPath $profilePath -Value $line
                Write-Host "Added the sne setup line to $profilePath"
            }
        }
    }

    if ($env:SNE_NOTIFY -eq '1') {
        & (Join-Path $installDir 'sne.exe') notify on
    }

    Write-Host ''
    Write-Host 'Done. Open a new terminal, then run:  sne install lts'
}

Install-Sne
