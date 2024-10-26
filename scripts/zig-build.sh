#!/bin/sh

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

pushd $SCRIPT_DIR/../examples/hello
cargo zigbuild --target aarch64-unknown-linux-ohos
cp ./dist/arm64-v8a/libhello.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
cp ./dist/index.d.ts $SCRIPT_DIR/../harmony-example/entry/src/main/cpp/types/libhello
popd

pushd $SCRIPT_DIR/../examples/napi
cargo zigbuild --target aarch64-unknown-linux-ohos
cp ./dist/arm64-v8a/libexample.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
cp ./dist/index.d.ts $SCRIPT_DIR/../harmony-example/entry/src/main/cpp/types/libexample
popd

pushd $SCRIPT_DIR/../examples/napi-compact-mode
cargo zigbuild --target aarch64-unknown-linux-ohos
cp ./dist/arm64-v8a/libcompact.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
popd

# build hap
# pushd $SCRIPT_DIR/../harmony-example
# hvigorw assembleApp --mode project -p product=default -p buildMode=debug --no-daemon

# install hap
# hdc install ./entry/build
# popd


# run test