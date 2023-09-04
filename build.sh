#!/bin/bash

# 定义目标
TARGETS=(
  # x86_64-pc-windows-gnu
  # x86_64-pc-windows-msvc
  # x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  # armv7-unknown-linux-gnueabihf
  x86_64-apple-darwin
  # aarch64-apple-darwin
)

# 遍历目标并构建
for target in ${TARGETS[@]}; do
  echo "Building for $target"
  cargo build --target $target --release
  cp "target/$target/release/rhfs" "rhfs-$target"
done