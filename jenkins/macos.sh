#!/usr/bin/env bash

set -euo pipefail
set -x

: "${VM_BUILDER_VERSION:?VM_BUILDER_VERSION environment variable is required}"
: "${FEENK_SIGNER_VERSION:?FEENK_SIGNER_VERSION environment variable is required}"
: "${TARGET:?TARGET environment variable is required}"
: "${APP_NAME:?APP_NAME environment variable is required}"
: "${APP_IDENTIFIER:?APP_IDENTIFIER environment variable is required}"
: "${APP_AUTHOR:?APP_AUTHOR environment variable is required}"
: "${APP_VERSION:?APP_VERSION environment variable is required}"
: "${APP_LIBRARIES:?APP_LIBRARIES environment variable is required}"
: "${APP_LIBRARIES_VERSIONS:?APP_LIBRARIES_VERSIONS environment variable is required}"
: "${APPLE_ID:?APPLE_ID environment variable is required}"
: "${APPLE_PASSWORD:?APPLE_PASSWORD environment variable is required}"
: "${CERT:?CERT environment variable is required}"
: "${CERT_PASSWORD:?CERT_PASSWORD environment variable is required}"
: "${SIGNING_IDENTITY:?SIGNING_IDENTITY environment variable is required}"
: "${VM_CLIENT_EXECUTABLE:?VM_CLIENT_EXECUTABLE environment variable is required}"

if [ -d target ]; then rm -Rf target; fi
if [ -d third_party ]; then rm -Rf third_party; fi
if [ -d libs ]; then rm -Rf libs; fi

git clean -fdx
git submodule foreach --recursive 'git fetch --tags'
git submodule update --init --recursive

rm -rf gtoolkit-vm-builder
curl -o gtoolkit-vm-builder -LsS "https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
chmod +x gtoolkit-vm-builder

curl -o feenk-signer -LsS "https://github.com/feenkcom/feenk-signer/releases/download/${FEENK_SIGNER_VERSION}/feenk-signer-${TARGET}"
chmod +x feenk-signer

# shellcheck disable=SC2086
./gtoolkit-vm-builder compile \
  --app-name "${APP_NAME}" \
  --identifier "${APP_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --icons icons/GlamorousToolkit.icns \
  --libraries ${APP_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose

# shellcheck disable=SC2086
./gtoolkit-vm-builder bundle \
  --strip-debug-symbols \
  --bundle-dir "bundle" \
  --app-name "${APP_NAME}" \
  --identifier "${APP_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --icons icons/GlamorousToolkit.icns \
  --libraries ${APP_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose
  
# shellcheck disable=SC2086
./gtoolkit-vm-builder bundle \
  --bundle-dir "bundle_with_debug_symbols" \
  --app-name "${APP_NAME}" \
  --identifier "${APP_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --icons icons/GlamorousToolkit.icns \
  --libraries ${APP_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose

cargo test --package vm-client-tests

./feenk-signer mac "bundle/${APP_NAME}.app"
./feenk-signer mac "bundle_with_debug_symbols/${APP_NAME}.app"

ditto -c -k --sequesterRsrc --keepParent "bundle/${APP_NAME}.app" "${APP_NAME}-${TARGET}.app.zip"
ditto -c -k --sequesterRsrc --keepParent "bundle_with_debug_symbols/${APP_NAME}.app" "${APP_NAME}-${TARGET}-with-debug-symbols.app.zip"

/Library/Developer/CommandLineTools/usr/bin/notarytool submit \
  --verbose \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "77664ZXL29" \
  --wait \
  "${APP_NAME}-${TARGET}.app.zip"

/Library/Developer/CommandLineTools/usr/bin/notarytool submit \
  --verbose \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "77664ZXL29" \
  --wait \
  "${APP_NAME}-${TARGET}-with-debug-symbols.app.zip"