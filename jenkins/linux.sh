#!/usr/bin/env bash

set -euo pipefail
set -x

: "${VM_BUILDER_VERSION:?VM_BUILDER_VERSION environment variable is required}"
: "${TARGET:?TARGET environment variable is required}"
: "${APP_NAME:?APP_NAME environment variable is required}"
: "${APP_IDENTIFIER:?APP_IDENTIFIER environment variable is required}"
: "${APP_PRO_IDENTIFIER:?APP_PRO_IDENTIFIER environment variable is required}"
: "${APP_AUTHOR:?APP_AUTHOR environment variable is required}"
: "${APP_VERSION:?APP_VERSION environment variable is required}"
: "${APP_LIBRARIES:?APP_LIBRARIES environment variable is required}"
: "${APP_PRO_LIBRARIES:?APP_PRO_LIBRARIES environment variable is required}"
: "${APP_LIBRARIES_VERSIONS:?APP_LIBRARIES_VERSIONS environment variable is required}"
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

echo "patchelf $(patchelf --version)"

# shellcheck disable=SC2086
./gtoolkit-vm-builder compile \
  --app-name "${APP_NAME}" \
  --identifier "${APP_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --libraries ${APP_LIBRARIES} ${APP_PRO_LIBRARIES} \
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
  --libraries ${APP_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose
  
# shellcheck disable=SC2086
./gtoolkit-vm-builder bundle \
  --strip-debug-symbols \
  --bundle-dir "bundle_pro" \
  --app-name "${APP_NAME}" \
  --identifier "${APP_PRO_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --libraries ${APP_LIBRARIES} ${APP_PRO_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose
  
# shellcheck disable=SC2086
./gtoolkit-vm-builder bundle \
  --bundle-dir "bundle_pro_with_debug_symbols" \
  --app-name "${APP_NAME}" \
  --identifier "${APP_PRO_IDENTIFIER}" \
  --author "${APP_AUTHOR}" \
  --version "${APP_VERSION}" \
  --libraries ${APP_LIBRARIES} ${APP_PRO_LIBRARIES} \
  --libraries-versions "${APP_LIBRARIES_VERSIONS}" \
  --release \
  --verbose

cargo test --package vm-client-tests

pushd "bundle/${APP_NAME}/"
zip -r "${APP_NAME}-${TARGET}.zip" .
popd
mv "bundle/${APP_NAME}-${TARGET}.zip" "./${APP_NAME}-${TARGET}.zip"

pushd "bundle_pro/${APP_NAME}/"
zip -r "${APP_NAME}-${TARGET}-pro.zip" .
popd
mv "bundle_pro/${APP_NAME}-${TARGET}-pro.zip" "./${APP_NAME}-${TARGET}-pro.zip"