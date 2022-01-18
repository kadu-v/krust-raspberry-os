#!/bin/sh


SERIALPORT0="/dev/pty0"
SERIALPORT1="/dev/pty1"
KERNEL_IMG="./chainloader/img/kernel8.img"

# host serial port
export SERIALPORT1="${SERIALPORT1}"


# make the kernel image
KERNEL_BIN=kernel8.img
cd chainloader
cargo objcopy --bin chainloader --release -- --strip-all -O binary ./img/kernel8.img &&
cd ..

# make a virtual serial port (boud rate is 115200)
sudo gnome-terminal -- bash -c "socat -d -d pty,b115200,raw,link=${SERIALPORT0},echo=0 pty,b115200,raw,link=${SERIALPORT1},echo=0"
sudo chown $USERNAME $SERIALPORT0 &&
sudo chown $USERNAME $SERIALPORT1 &&

# open a new terminal window and litsen a serial port of guset device from host serial port
# gnome-terminal -- bash -c "cat < ${SERIALPORT1}" &&

# open a new terminal window for sending a kernel binary
# gnome-terminal -- bash -c "bash" &&
stty -F ${SERIALPORT0} 115200 &&
stty -F ${SERIALPORT1} 115200 &&
cd rminipush &&
gnome-terminal -- bash -c "stty -F ${SERIALPORT1};cargo run;bash" &&
cd .. &&

sleep 3
echo "start booting..."
# launch qemu-system-aarch64
# connect a /dev/pty0 to use a uart
qemu-system-aarch64 -M raspi3 -serial $SERIALPORT0 -display none -kernel $KERNEL_IMG
# qemu-system-aarch64 -M raspi3b -serial stdio -display none -kernel ./img/kernel8.img

