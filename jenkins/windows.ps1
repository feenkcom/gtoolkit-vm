$ErrorActionPreference = 'Stop'      # like set -e + pipefail

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

$appLibraries = $env:APP_LIBRARIES -split '[,;\s]+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

Remove-Item -Force -Recurse -Path target -ErrorAction Ignore
Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore
Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore

git clean -fdx
git submodule foreach --recursive 'git fetch --tags'
git submodule update --init --recursive

Remove-Item gtoolkit-vm-builder.exe -ErrorAction Ignore
curl -o gtoolkit-vm-builder.exe "https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/$env:VM_BUILDER_VERSION/gtoolkit-vm-builder-$env:HOST.exe"


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

Compress-Archive -Path "target/$env:TARGET/release/bundle/$env:APP_NAME/*" -DestinationPath "$env:APP_NAME-$env:TARGET.zip"
