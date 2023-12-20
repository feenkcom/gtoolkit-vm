#!/usr/bin/env bash
# Make sure that iOS simulator is running

# exit when any command fails
set -e
set -o xtrace

BIN_NAME="vm_client-cli"
FOLDER="vm-client"

PROFILE="release"
BUILD_TARGET="aarch64-apple-ios-sim"

BUNDLE_DIR="target/${BUILD_TARGET}/${PROFILE}/bundle/ios/${BIN_NAME}.app"
DYLIB_DIR="Plugins"

rm -rf ${BUNDLE_DIR}/${DYLIB_DIR}
mkdir ${BUNDLE_DIR}/${DYLIB_DIR}

cp -rf target/${BUILD_TARGET}/${PROFILE}/*.dylib ${BUNDLE_DIR}/${DYLIB_DIR}/

for lib in ${BUNDLE_DIR}/${DYLIB_DIR}/*.dylib; do
  ./rpath --lib "${lib}" --path ${DYLIB_DIR}
done

export SIMCTL_CHILD_RUST_LOG="error"

xcrun simctl install booted target/${BUILD_TARGET}/${PROFILE}/bundle/ios/${BIN_NAME}.app
xcrun simctl launch \
  --console \
  --terminate-running-process \
  booted com.feenkcom.${BIN_NAME} \
  --worker yes --interactive GlamorousToolkit.image