$builder = 'gtoolkit-vm-builder.exe'

#If the builder does not exist, create it.
if (-not(Test-Path -Path $builder -PathType Leaf)) {
    try {
        curl -o $builder https://github.com/feenkcom/gtoolkit-vm-builder/releases/latest/download/gtoolkit-vm-builder-x86_64-pc-windows-msvc.exe
    }
    catch {
        throw $_.Exception.Message
    }
}

& .\$builder `
    --release `
    --target aarch64-pc-windows-msvc `
    -vvvv `
    --app-name 'GlamorousToolkit' `
    --identifier 'com.gtoolkit' `
    --author "feenk gmbh <contact@feenk.com>" `
    --libraries-versions libraries.version `
    --executables cli `
    --icons icons/GlamorousToolkit.ico