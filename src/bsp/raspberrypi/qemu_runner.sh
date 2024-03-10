#!/bin/sh


cargo objcopy --bin kernel --release -- --strip-all -O binary ./img/kernel8.img &&

# qemu-system-aarch64 -M raspi3b -serial stdio -display none -kernel ./img/kernel8.img
qemu-system-aarch64 -M raspi3b -serial stdio -kernel ./img/kernel8.img