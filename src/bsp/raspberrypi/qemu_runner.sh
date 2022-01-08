#!/bin/sh

# make the kernel image
KERNEL_BIN=kernel8.img
cargo objcopy --bin kernel -- --strip-all -O binary ./img/kernel8.img &&

# laounch qemu
qemu-system-aarch64 -M raspi3ap -d in_asm -display none -kernel ./img/kernel8.img

