$ErrorActionPreference = 'Stop'      # like set -e + pipefail

# fail on ANY external command automatically
Register-EngineEvent PowerShell.Exiting -Action {
    if (\$global:LASTEXITCODE -ne 0) {
        exit \$global:LASTEXITCODE
    }
}

##
# Validate required environment variables early and expose them as script variables.
$requiredVariables = @(
    'VM_BUILDER_VERSION',
    'TARGET',
    'HOST',
    'APP_NAME',
    'APP_IDENTIFIER',
    'APP_AUTHOR',
    'APP_VERSION',
    'APP_LIBRARIES',
    'APP_LIBRARIES_VERSIONS',
    'VM_CLIENT_EXECUTABLE'
)

foreach ($name in $requiredVariables) {
    $envValue = [Environment]::GetEnvironmentVariable($name)

    if ([string]::IsNullOrWhiteSpace($envValue)) {
        Write-Error "Missing required environment variable: $name"
        exit 1
    }
}

Write-Host "VM_BUILDER_VERSION=$env:VM_BUILDER_VERSION"
Write-Host "TARGET=$env:TARGET"
Write-Host "HOST=$env:HOST"
Write-Host "APP_VERSION=$env:APP_VERSION"
Write-Host "PWD=$(Get-Location)"
Write-Host "PowerShell=$($PSVersionTable.PSVersion)"
Write-Host "OS=$([System.Runtime.InteropServices.RuntimeInformation]::OSDescription)"
Write-Host "ProcessArch=$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)"

$appLibraries = $env:APP_LIBRARIES -split '[,;\s]+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

Remove-Item -Force -Recurse -Path target -ErrorAction Ignore
Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore
Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore

git clean -fdx
git submodule foreach --recursive 'git fetch --tags'
git submodule update --init --recursive

$builder = 'gtoolkit-vm-builder.exe'
$builderUrl = "https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/$env:VM_BUILDER_VERSION/gtoolkit-vm-builder-$env:HOST.exe"

Remove-Item $builder -ErrorAction Ignore
Write-Host "Downloading builder from $builderUrl"
curl.exe --fail --location --show-error --output $builder $builderUrl

if (-not (Test-Path -Path $builder -PathType Leaf)) {
    throw "Builder was not downloaded: $builder"
}

# validate the gtoolkit-vm-builder before executing it:
$builderItem = Get-Item $builder
Write-Host "Builder path: $($builderItem.FullName)"
Write-Host "Builder size: $($builderItem.Length) bytes"
Write-Host "Builder sha256: $((Get-FileHash $builder -Algorithm SHA256).Hash)"

if ($builderItem.Length -lt 1MB) {
    Write-Host "First bytes of suspicious builder:"
    Format-Hex -Path $builder -Count 256
    throw "Builder is unexpectedly small. This is likely an error page or wrong release asset."
}

./gtoolkit-vm-builder.exe compile `
    --app-name $env:APP_NAME `
    --identifier $env:APP_IDENTIFIER `
    --author $env:APP_AUTHOR `
    --version $env:APP_VERSION `
    --libraries $appLibraries `
    --libraries-versions $env:APP_LIBRARIES_VERSIONS `
    --icons icons/GlamorousToolkit.ico `
    --release `
    --target $env:TARGET `
    --verbose

./gtoolkit-vm-builder.exe bundle `
    --strip-debug-symbols `
    --bundle-dir "bundle" `
    --app-name $env:APP_NAME `
    --identifier $env:APP_IDENTIFIER `
    --author $env:APP_AUTHOR `
    --version $env:APP_VERSION `
    --libraries $appLibraries `
    --libraries-versions $env:APP_LIBRARIES_VERSIONS `
    --icons icons/GlamorousToolkit.ico `
    --release `
    --target $env:TARGET `
    --verbose

./gtoolkit-vm-builder.exe bundle `
    --bundle-dir "bundle_with_debug_symbols" `
    --app-name $env:APP_NAME `
    --identifier $env:APP_IDENTIFIER `
    --author $env:APP_AUTHOR `
    --version $env:APP_VERSION `
    --libraries $appLibraries `
    --libraries-versions $env:APP_LIBRARIES_VERSIONS `
    --icons icons/GlamorousToolkit.ico `
    --release `
    --target $env:TARGET `
    --verbose

cargo test --package vm-client-tests

Compress-Archive -Path "bundle/$env:APP_NAME/*" -DestinationPath "$env:APP_NAME-$env:TARGET.zip"
Compress-Archive -Path "bundle_with_debug_symbols/$env:APP_NAME/*" -DestinationPath "$env:APP_NAME-$env:TARGET-with-debug-symbols.zip"
