# Writes the package manager manifests for a published sne release:
#
#   bucket/sne.json     Scoop (this repository is the bucket)
#   Formula/sne.rb      Homebrew (this repository is the tap)
#   packaging/aur/      PKGBUILD and .SRCINFO for the AUR package sne-bin
#   packaging/winget/   manifests for a pull request to microsoft/winget-pkgs
#
# Run it from anywhere after the release is published, then commit the result:
#
#   pwsh packaging/update-manifests.ps1 0.1.0-beta.4
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

# The AUR package also installs the profile script, taken from the tagged source.
$profileUrl = "https://raw.githubusercontent.com/sne-sh/sne/$Version/packaging/profile.sh"
$profileBytes = (Invoke-WebRequest $profileUrl).Content
if ($profileBytes -is [string]) { $profileBytes = [Text.Encoding]::UTF8.GetBytes($profileBytes) }
$profileSum = ([Security.Cryptography.SHA256]::HashData([byte[]]$profileBytes) | ForEach-Object { $_.ToString('x2') }) -join ''

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
    X86   = "sne-$Version-i686-pc-windows-msvc"
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
        "32bit": {
            "url": "{{DOWNLOADS}}/{{X86}}.zip",
            "hash": "{{X86_SUM}}",
            "extract_dir": "{{X86}}"
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
        X86_SUM   = Sum "$($win.X86).zip"
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

# pacman versions cannot contain a hyphen. Same form as the Arch package in the release.
$aur = @{
    PKGVER      = $Version.Replace('-', '')
    PROFILE_SUM = $profileSum
    X64_SUM     = Sum "sne-$Version-x86_64-unknown-linux-musl.tar.gz"
    ARM64_SUM   = Sum "sne-$Version-aarch64-unknown-linux-musl.tar.gz"
}

Write-Manifest 'packaging/aur/PKGBUILD' @'
# Maintainer: sne contributors <sne-sh@users.noreply.github.com>
# Written by packaging/update-manifests.ps1 in https://github.com/sne-sh/sne. Do not edit by hand.
pkgname=sne-bin
_tag={{VERSION}}
pkgver={{PKGVER}}
pkgrel=1
pkgdesc="Simple Node Environment: a fast Node.js version manager"
arch=('x86_64' 'aarch64')
url="https://github.com/sne-sh/sne"
license=('MIT')
provides=('sne')
conflicts=('sne')
source=("sne-profile-$_tag.sh::https://raw.githubusercontent.com/sne-sh/sne/$_tag/packaging/profile.sh")
source_x86_64=("https://github.com/sne-sh/sne/releases/download/$_tag/sne-$_tag-x86_64-unknown-linux-musl.tar.gz")
source_aarch64=("https://github.com/sne-sh/sne/releases/download/$_tag/sne-$_tag-aarch64-unknown-linux-musl.tar.gz")
sha256sums=('{{PROFILE_SUM}}')
sha256sums_x86_64=('{{X64_SUM}}')
sha256sums_aarch64=('{{ARM64_SUM}}')

package() {
  cd "sne-$_tag-$CARCH-unknown-linux-musl"
  install -Dm755 sne "$pkgdir/usr/bin/sne"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  # Puts the default Node.js version on PATH for login shells.
  install -Dm644 "$srcdir/sne-profile-$_tag.sh" "$pkgdir/etc/profile.d/sne.sh"
}
'@ $aur

# What `makepkg --printsrcinfo` prints for the PKGBUILD above. The Packages workflow checks it.
$tab = "`t"
Write-Manifest 'packaging/aur/.SRCINFO' (@'
pkgbase = sne-bin
>pkgdesc = Simple Node Environment: a fast Node.js version manager
>pkgver = {{PKGVER}}
>pkgrel = 1
>url = https://github.com/sne-sh/sne
>arch = x86_64
>arch = aarch64
>license = MIT
>provides = sne
>conflicts = sne
>source = sne-profile-{{VERSION}}.sh::https://raw.githubusercontent.com/sne-sh/sne/{{VERSION}}/packaging/profile.sh
>sha256sums = {{PROFILE_SUM}}
>source_x86_64 = {{DOWNLOADS}}/sne-{{VERSION}}-x86_64-unknown-linux-musl.tar.gz
>sha256sums_x86_64 = {{X64_SUM}}
>source_aarch64 = {{DOWNLOADS}}/sne-{{VERSION}}-aarch64-unknown-linux-musl.tar.gz
>sha256sums_aarch64 = {{ARM64_SUM}}

pkgname = sne-bin
'@ -replace '(?m)^>', $tab) $aur

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
Commands:
- sne
ReleaseDate: {{DATE}}
Installers:
- Architecture: x64
  InstallerUrl: {{DOWNLOADS}}/sne-{{VERSION}}-x64-setup.exe
  InstallerSha256: {{X64_SUM}}
- Architecture: x86
  InstallerUrl: {{DOWNLOADS}}/sne-{{VERSION}}-x86-setup.exe
  InstallerSha256: {{X86_SUM}}
- Architecture: arm64
  InstallerUrl: {{DOWNLOADS}}/sne-{{VERSION}}-arm64-setup.exe
  InstallerSha256: {{ARM64_SUM}}
ManifestType: installer
ManifestVersion: 1.10.0
'@ @{
    X64_SUM   = (Sum "sne-$Version-x64-setup.exe").ToUpperInvariant()
    X86_SUM   = (Sum "sne-$Version-x86-setup.exe").ToUpperInvariant()
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
