#!/bin/bash

./gtoolkit-vm-builder \
    --release \
    --app-name 'GlamorousToolkit' \
    --identifier 'com.gtoolkit' \
    --author "feenk gmbh <contact@feenk.com>" \
    --libraries-versions libraries.version \
    --icons icons/GlamorousToolkit.icns \
    --libraries boxer cairo clipboard crypto freetype git gleam glutin process sdl2 skia winit test-library