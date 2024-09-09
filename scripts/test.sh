#!/bin/sh

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

pushd $SCRIPT_DIR/../examples/hello
ohrs build
cp ./dist/arm64-v8a/libhello.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
cp ./dist/index.d.ts $SCRIPT_DIR/../harmony-example/entry/src/main/cpp/types/libhello
popd

pushd $SCRIPT_DIR/../examples/napi
ohrs build
cp ./dist/arm64-v8a/libnapi.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
cp ./dist/index.d.ts $SCRIPT_DIR/../harmony-example/entry/src/main/cpp/types/libnapi
popd

pushd $SCRIPT_DIR/../examples/napi-compact-mode
ohrs build
cp ./dist/arm64-v8a/libnapi_compact_mode.so $SCRIPT_DIR/../harmony-example/entry/libs/arm64-v8a
popd

# build hap
pushd $SCRIPT_DIR/../harmony-example
hvigorw assembleApp --mode project -p product=default -p buildMode=debug --no-daemon

# install hap
hdc install ./entry/build
popd


# run test