#!/usr/bin/env bash

set -e

TARGET="riscv32im-unknown-none-elf"
CRATE_NAME="$(basename "$(pwd)")"
PROFILE="debug"

if [ "$1" = "--release" ]; then
    PROFILE="release"
fi

ELF="target/${TARGET}/${PROFILE}/${CRATE_NAME}"
OUT="build"
PREFIX="riscv64-unknown-elf"

mkdir -p "${OUT}"

rm -f "${OUT}/${CRATE_NAME}.bin" "${OUT}/${CRATE_NAME}.hex" "${OUT}/${CRATE_NAME}.txt"

"${PREFIX}-objcopy" -O binary  "${ELF}" "${OUT}/${CRATE_NAME}.bin"
"${PREFIX}-objcopy" -O verilog "${ELF}" "${OUT}/${CRATE_NAME}.hex"
sed -i 's/@30000000/@00000000/g' "${OUT}/${CRATE_NAME}.hex"
"${PREFIX}-objdump" -d "${ELF}" > "${OUT}/${CRATE_NAME}.txt"
