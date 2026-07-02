#!/bin/sh


mkdir -p ./img &&

cargo objcopy --bin kernel --release -- --strip-all -O binary ./img/kernel8.img &&

# qemu-system-aarch64 -M raspi4b -serial stdio -display none -kernel ./img/kernel8.img
qemu-system-aarch64 -M raspi4b -serial stdio -kernel ./img/kernel8.img