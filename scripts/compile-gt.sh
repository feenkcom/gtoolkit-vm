#!/usr/bin/env bash

BUILDER="gtoolkit-vm-builder"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/download-vm-builder.sh" "$BUILDER"

export CARGO_LOG=cargo::core::compiler::fingerprint=info

"./$BUILDER" compile \
  --release \
  --app-name 'GlamorousToolkit' \
  --identifier 'com.gtoolkit' \
  --author "feenk gmbh <contact@feenk.com>" \
  --libraries-versions libraries.version \
  --icons icons/GlamorousToolkit.icns \
  --executables app cli \
  --libraries boxer cairo clipboard crypto filewatcher freetype git gleam glutin pixels process sdl2 skia ssl webview winit winit30 test-library