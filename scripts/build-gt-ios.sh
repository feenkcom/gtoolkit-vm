#!/usr/bin/env bash
# Make sure that iOS simulator is running
# To create simulator:
# xcrun simctl create iPad com.apple.CoreSimulator.SimDeviceType.iPad-Pro-12-9-inch-6th-generation-8GB com.apple.CoreSimulator.SimRuntime.iOS-16-2
# 43722D7D-2CD6-458D-ADAC-194D49D3294C
# Boot simulator:
# xcrun simctl boot 43722D7D-2CD6-458D-ADAC-194D49D3294C
# Shutdown simulator
# xcrun simctl shutdown 43722D7D-2CD6-458D-ADAC-194D49D3294C
# Open simulator app:
# open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app/


# exit when any command fails
set -e
set -o xtrace

BIN_NAME="vm_client-cli"
FOLDER="vm-client"

PROFILE="release"
BUILD_TARGET="aarch64-apple-ios-sim"

cd ${FOLDER}
cargo bundle --bin ${BIN_NAME} --${PROFILE} --target ${BUILD_TARGET} --no-default-features --features full
echo "Created bundle ${BIN_NAME} for ${BUILD_TARGET}"
cd ..

BUNDLE_DIR="target/${BUILD_TARGET}/${PROFILE}/bundle/ios/${BIN_NAME}.app"
DYLIB_DIR="Plugins"

./rpath --lib "${BUNDLE_DIR}/${BIN_NAME}" --path ${DYLIB_DIR}