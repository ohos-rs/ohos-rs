#!/bin/sh
exec $OHOS_NDK_HOME/native/llvm/bin/clang \
  -target arm-linux-ohos \
  --sysroot=$OHOS_NDK_HOME/native/sysroot \
  -D__MUSL__ \
  -march=armv7-a \
  -mfloat-abi=softfp \
  -mtune=generic-armv7-a \
  -mthumb \
  "$@"
