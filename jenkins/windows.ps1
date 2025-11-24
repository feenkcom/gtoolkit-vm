$ErrorActionPreference = 'Stop'      # like set -e + pipefail
Set-PSDebug -Trace 1                 # like set -x

##
# Validate required environment variables early and expose them as script variables.
$requiredVariables = @(
    'VM_BUILDER_VERSION',
    'TARGET',
    'APP_NAME',
    'APP_IDENTIFIER',
    'APP_AUTHOR',
    'APP_VERSION',
    'APP_LIBRARIES',
    'APP_LIBRARIES_VERSIONS',
    'VM_CLIENT_EXECUTABLE'
)

foreach ($name in $requiredVariables) {
    $variableValue = Get-Variable -Name $name -ValueOnly -ErrorAction SilentlyContinue
    $envValue = [Environment]::GetEnvironmentVariable($name)
    $value = if (-not [string]::IsNullOrWhiteSpace($variableValue)) { $variableValue } else { $envValue }

    if ([string]::IsNullOrWhiteSpace($value)) {
        Write-Error "Missing required environment variable: $name"
        exit 1
    }

    Set-Variable -Name $name -Value $value -Scope Script
}

$appLibraries = $APP_LIBRARIES -split '[,;\s]+' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

Remove-Item -Force -Recurse -Path target -ErrorAction Ignore
Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore
Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore

git clean -fdx
git submodule foreach --recursive 'git fetch --tags'
git submodule update --init --recursive

Remove-Item gtoolkit-vm-builder.exe -ErrorAction Ignore
curl -o gtoolkit-vm-builder.exe "https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}.exe"


./gtoolkit-vm-builder.exe compile `
    --app-name ${APP_NAME} `
    --identifier ${APP_IDENTIFIER} `
    --author ${APP_AUTHOR} `
    --version ${APP_VERSION} `
    --libraries $appLibraries `
    --libraries-versions ${APP_LIBRARIES_VERSIONS} `
    --icons icons/GlamorousToolkit.ico `
    --release `
    --target ${TARGET} `
    --verbose

./gtoolkit-vm-builder.exe bundle `
    --strip-debug-symbols `
    --app-name ${APP_NAME} `
    --identifier ${APP_IDENTIFIER} `
    --author ${APP_AUTHOR} `
    --version ${APP_VERSION} `
    --libraries $appLibraries `
    --libraries-versions ${APP_LIBRARIES_VERSIONS} `
    --icons icons/GlamorousToolkit.ico `
    --release `
    --target ${TARGET} `
    --verbose

cargo test --package vm-client-tests

Compress-Archive -Path "target/${TARGET}/release/bundle/${APP_NAME}/*" -DestinationPath "${APP_NAME}-${TARGET}.zip"
