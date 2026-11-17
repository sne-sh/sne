# Writes the package manager manifests for a published sne release:
#
#   bucket/sne.json     Scoop (this repository is the bucket)
#   Formula/sne.rb      Homebrew (this repository is the tap)
#   packaging/winget/   manifests for a pull request to microsoft/winget-pkgs
#
# Run it from anywhere after the release is published, then commit the result:
#
#   pwsh packaging/update-manifests.ps1 0.1.0
#
# The checksums come from the release's SHA256SUMS.txt, so the release must be published first.

param(
    [Parameter(Mandatory)][string]$Version
)

$ErrorActionPreference = 'Stop'
$repo = 'https://github.com/sne-sh/sne'
$root = Split-Path -Parent $PSScriptRoot
$downloads = "$repo/releases/download/$Version"
$description = 'Simple Node Environment: a fast, cross-platform Node.js version manager'

$sums = @{}
foreach ($line in (Invoke-RestMethod "$downloads/SHA256SUMS.txt") -split "`n") {
    $hash, $name = $line.Trim() -split '\s+', 2
    if ($name) { $sums[$name.TrimStart('*')] = $hash.ToLowerInvariant() }
}
function Sum([string]$file) {
    if (-not $sums.ContainsKey($file)) { throw "$file is not listed in SHA256SUMS.txt of $Version" }
    $sums[$file]
}

$released = try {
    ([datetime](Invoke-RestMethod "https://api.github.com/repos/sne-sh/sne/releases/tags/$Version").published_at).ToUniversalTime()
} catch { [datetime]::UtcNow }

# Fills in {{NAME}} placeholders and writes the file with LF line endings and no byte order mark.
function Write-Manifest([string]$path, [string]$template, [hashtable]$extra = @{}) {
    $values = @{
        VERSION     = $Version
        DOWNLOADS   = $downloads
        REPO        = $repo
        DESCRIPTION = $description
        DATE        = $released.ToString('yyyy-MM-dd')
    } + $extra
    $text = $template
    foreach ($key in $values.Keys) { $text = $text.Replace("{{$key}}", $values[$key]) }
    if ($text -match '\{\{[A-Z_0-9]+\}\}') { throw "${path}: $($Matches[0]) was not filled in" }
    $file = Join-Path $root $path
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $file) | Out-Null
    [IO.File]::WriteAllText($file, ($text -replace "`r`n", "`n").TrimEnd() + "`n")
    Write-Host "wrote $path"
}

$win = @{
    X64   = "sne-$Version-x86_64-pc-windows-msvc"
    ARM64 = "sne-$Version-aarch64-pc-windows-msvc"
}

Write-Manifest 'bucket/sne.json' @'
{
    "version": "{{VERSION}}",
    "description": "{{DESCRIPTION}}",
    "homepage": "https://sne-sh.github.io/sne/",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "{{DOWNLOADS}}/{{X64}}.zip",
            "hash": "{{X64_SUM}}",
            "extract_dir": "{{X64}}"
        },
        "arm64": {
            "url": "{{DOWNLOADS}}/{{ARM64}}.zip",
            "hash": "{{ARM64_SUM}}",
            "extract_dir": "{{ARM64}}"
        }
    },
    "bin": "sne.exe",
    "notes": [
        "Run `sne install lts` to install Node.js. It works in every terminal with no more setup.",
        "To switch versions per terminal, add the setup line from `sne env --help` to your profile."
    ]
}
'@ ($win + @{
        X64_SUM   = Sum "$($win.X64).zip"
        ARM64_SUM = Sum "$($win.ARM64).zip"
    })

Write-Manifest 'Formula/sne.rb' @'
# Written by packaging/update-manifests.ps1. Do not edit by hand.
class Sne < Formula
  desc "Simple Node Environment: a fast Node.js version manager"
  homepage "https://sne-sh.github.io/sne/"
  version "{{VERSION}}"
  license "MIT"

  on_macos do
    on_arm do
      url "{{DOWNLOADS}}/sne-{{VERSION}}-aarch64-apple-darwin.tar.gz"
      sha256 "{{MAC_ARM}}"
    end
    on_intel do
      url "{{DOWNLOADS}}/sne-{{VERSION}}-x86_64-apple-darwin.tar.gz"
      sha256 "{{MAC_X64}}"
    end
  end

  on_linux do
    on_arm do
      url "{{DOWNLOADS}}/sne-{{VERSION}}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "{{LINUX_ARM}}"
    end
    on_intel do
      url "{{DOWNLOADS}}/sne-{{VERSION}}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "{{LINUX_X64}}"
    end
  end

  def install
    bin.install "sne"
  end

  def caveats
    <<~EOS
      Add the setup line to your shell profile, then open a new terminal:
        bash, zsh:  eval "$(sne env --use-on-cd)"
        fish:       sne env --use-on-cd | source
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/sne --version")
  end
end
'@ @{
    MAC_ARM   = Sum "sne-$Version-aarch64-apple-darwin.tar.gz"
    MAC_X64   = Sum "sne-$Version-x86_64-apple-darwin.tar.gz"
    LINUX_ARM = Sum "sne-$Version-aarch64-unknown-linux-musl.tar.gz"
    LINUX_X64 = Sum "sne-$Version-x86_64-unknown-linux-musl.tar.gz"
}

# winget wants one folder per version: manifests/s/sne-sh/sne/<version>/ in microsoft/winget-pkgs.
$winget = "packaging/winget/manifests/s/sne-sh/sne"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $root 'packaging/winget/manifests')

Write-Manifest "$winget/$Version/sne-sh.sne.yaml" @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.10.0.schema.json

PackageIdentifier: sne-sh.sne
PackageVersion: {{VERSION}}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.10.0
'@

Write-Manifest "$winget/$Version/sne-sh.sne.installer.yaml" @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.10.0.schema.json

PackageIdentifier: sne-sh.sne
PackageVersion: {{VERSION}}
InstallerType: inno
Scope: user
InstallModes:
- interactive
- silent
- silentWithProgress
UpgradeBehavior: install
# Only the PATH task: a silent install leaves the PowerShell profile and Task Scheduler alone.
InstallerSwitches:
  Custom: /TASKS=addtopath
Commands:
- sne
ReleaseDate: {{DATE}}
Installers:
- Architecture: x64
  InstallerUrl: {{DOWNLOADS}}/sne-{{VERSION}}-x64-setup.exe
  InstallerSha256: {{X64_SUM}}
- Architecture: arm64
  InstallerUrl: {{DOWNLOADS}}/sne-{{VERSION}}-arm64-setup.exe
  InstallerSha256: {{ARM64_SUM}}
ManifestType: installer
ManifestVersion: 1.10.0
'@ @{
    X64_SUM   = (Sum "sne-$Version-x64-setup.exe").ToUpperInvariant()
    ARM64_SUM = (Sum "sne-$Version-arm64-setup.exe").ToUpperInvariant()
}

Write-Manifest "$winget/$Version/sne-sh.sne.locale.en-US.yaml" @'
# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.10.0.schema.json

PackageIdentifier: sne-sh.sne
PackageVersion: {{VERSION}}
PackageLocale: en-US
Publisher: sne contributors
PublisherUrl: {{REPO}}
PublisherSupportUrl: {{REPO}}/issues
PackageName: sne
PackageUrl: https://sne-sh.github.io/sne/
License: MIT
LicenseUrl: {{REPO}}/blob/{{VERSION}}/LICENSE
ShortDescription: A fast, cross-platform Node.js version manager
Description: |-
  sne (Simple Node Environment) installs Node.js versions from nodejs.org and switches between them,
  per terminal or for every terminal. It reads .nvmrc, .node-version and package.json, checks the
  SHA-256 of every download, and needs no administrator rights.
Moniker: sne
Tags:
- node
- nodejs
- nvm
- version-manager
- cli
ReleaseNotesUrl: {{REPO}}/releases/tag/{{VERSION}}
ManifestType: defaultLocale
ManifestVersion: 1.10.0
'@
