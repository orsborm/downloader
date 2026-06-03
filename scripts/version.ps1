# Version Management Script
# Usage:
#   .\scripts\version.ps1                # Show current version
#   .\scripts\version.ps1 patch          # 1.0.0 -> 1.0.1
#   .\scripts\version.ps1 minor          # 1.0.0 -> 1.1.0
#   .\scripts\version.ps1 major          # 1.0.0 -> 2.0.0
#   .\scripts\version.ps1 set 1.2.3      # Set specific version

param(
    [string]$Action = "show",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$PackageJson = Join-Path $PSScriptRoot "..\package.json"
$TauriConf = Join-Path $PSScriptRoot "..\src-tauri\tauri.conf.json"
$CargoToml = Join-Path $PSScriptRoot "..\src-tauri\Cargo.toml"

# UTF-8 encoding without BOM (PS 5.1 -Encoding UTF8 adds BOM)
$Utf8NoBom = New-Object System.Text.UTF8Encoding $false

function Get-CurrentVersion {
    $pkg = Get-Content $PackageJson -Raw | ConvertFrom-Json
    return $pkg.version
}

function Set-Version {
    param([string]$NewVersion)

    if ($NewVersion -notmatch '^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$') {
        Write-Error "Invalid version format: $NewVersion (expected x.y.z)"
        return
    }

    Write-Host "Updating version to: $NewVersion" -ForegroundColor Cyan

    # 1. package.json
    $pkg = Get-Content $PackageJson -Raw | ConvertFrom-Json
    $pkg.version = $NewVersion
    [System.IO.File]::WriteAllText($PackageJson, ($pkg | ConvertTo-Json -Depth 10), $Utf8NoBom)
    Write-Host "  OK package.json" -ForegroundColor Green

    # 2. tauri.conf.json
    $conf = Get-Content $TauriConf -Raw | ConvertFrom-Json
    $conf.version = $NewVersion
    [System.IO.File]::WriteAllText($TauriConf, ($conf | ConvertTo-Json -Depth 10), $Utf8NoBom)
    Write-Host "  OK tauri.conf.json" -ForegroundColor Green

    # 3. Cargo.toml
    $cargo = Get-Content $CargoToml -Raw
    $cargo = $cargo -replace '(?m)^version\s*=\s*"[^"]*"', "version = `"$NewVersion`""
    [System.IO.File]::WriteAllText($CargoToml, $cargo, $Utf8NoBom)
    Write-Host "  OK Cargo.toml" -ForegroundColor Green

    Write-Host ""
    Write-Host "All versions updated to $NewVersion" -ForegroundColor Green
}

function Bump-Version {
    param([string]$Part)

    $current = Get-CurrentVersion
    $parts = $current -split '\.'
    if ($parts.Count -lt 3) {
        Write-Error "Invalid version format: $current"
        return
    }

    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]

    switch ($Part) {
        "major" { $major++; $minor = 0; $patch = 0 }
        "minor" { $minor++; $patch = 0 }
        "patch" { $patch++ }
        default {
            Write-Error "Unknown bump type: $Part (expected major/minor/patch)"
            return
        }
    }

    $newVersion = "$major.$minor.$patch"
    Set-Version $newVersion
}

switch ($Action) {
    "show" {
        $v = Get-CurrentVersion
        Write-Host "Current version: v$v" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Config files:" -ForegroundColor Yellow
        Write-Host "  package.json:     $v"
        Write-Host "  tauri.conf.json:  $v"
        $cargo = Get-Content $CargoToml -Raw
        if ($cargo -match 'version\s*=\s*"([^"]*)"') {
            Write-Host "  Cargo.toml:       $($Matches[1])"
        }
    }
    "major" { Bump-Version "major" }
    "minor" { Bump-Version "minor" }
    "patch" { Bump-Version "patch" }
    "set" {
        if ([string]::IsNullOrEmpty($Version)) {
            Write-Error "Please specify version, e.g.: .\scripts\version.ps1 set 1.2.3"
            return
        }
        Set-Version $Version
    }
    default {
        Write-Host "Usage:" -ForegroundColor Yellow
        Write-Host "  .\scripts\version.ps1              Show current version"
        Write-Host "  .\scripts\version.ps1 patch         Patch bump 1.0.0 -> 1.0.1"
        Write-Host "  .\scripts\version.ps1 minor         Minor bump 1.0.0 -> 1.1.0"
        Write-Host "  .\scripts\version.ps1 major         Major bump 1.0.0 -> 2.0.0"
        Write-Host "  .\scripts\version.ps1 set 1.2.3     Set specific version"
    }
}
