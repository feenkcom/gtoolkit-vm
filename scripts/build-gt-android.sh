#!/usr/bin/env bash

set -euo pipefail
set -x

BUILDER="gtoolkit-vm-builder"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

IMAGE_ARCHIVE_URL="https://github.com/feenkcom/gtoolkit/releases/latest/download/GlamorousToolkit-MacOS-aarch64-v1.1.18.zip"
IMAGE_ARCHIVE_NAME="GlamorousToolkit.zip"
IMAGE_ARCHIVE_PATH="target/assets"
IMAGE_ARCHIVE_FILE="$IMAGE_ARCHIVE_PATH/$IMAGE_ARCHIVE_NAME"

"$SCRIPT_DIR/download-vm-builder.sh" "$BUILDER"

if [[ ! -f "$IMAGE_ARCHIVE_FILE" ]]; then
  echo "Downloading GlamorousToolkit image archive..."
  mkdir -p "$IMAGE_ARCHIVE_PATH"
  if ! curl -L --fail -o "$IMAGE_ARCHIVE_FILE" "$IMAGE_ARCHIVE_URL"; then
    echo "Failed to download GlamorousToolkit image archive" >&2
    exit 1
  fi
else
  echo "Using existing archive at $IMAGE_ARCHIVE_FILE"
fi

export CARGO_LOG=cargo::core::compiler::fingerprint=info

"./$BUILDER" build \
  --release \
  --bundle-dir "$(pwd)/target/bundle" \
  --app-name 'GlamorousToolkit' \
  --identifier 'com.gtoolkit' \
  --author "feenk gmbh <contact@feenk.com>" \
  --libraries-versions libraries.version \
  --icons "icons/android" \
  --executables android \
  --target aarch64-linux-android \
  -vvvv \
  --libraries clipboard filewatcher pixels process skia winit winit30 webview crypto git ssl

cd target || exit
zip -r bundle/GlamorousToolkit.apk assets/
"$ANDROID_HOME"/build-tools/34.0.0-rc1/zipalign -f 4 bundle/GlamorousToolkit.apk bundle/GlamorousToolkit-aligned.apk
"$ANDROID_HOME"/build-tools/34.0.0-rc1/apksigner sign --ks ~/.android/debug.keystore --ks-pass pass:android bundle/GlamorousToolkit-aligned.apk
adb install -r -d bundle/GlamorousToolkit-aligned.apk