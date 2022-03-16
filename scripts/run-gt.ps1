# Make sure to create a local installation of GT using the following script:
# wget https://dl.feenk.com/scripts/windows.ps1 -OutFile windows.ps1; ./windows.ps1

$vm = '.\..\target\x86_64-pc-windows-msvc\release\bundle\GlamorousToolkit\bin\GlamorousToolkit-cli.exe'
$image = 'GlamorousToolkit.image'

Push-Location .\glamoroustoolkit

& $vm $image eval ""