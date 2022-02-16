
rustup override set nightly-2021-01-07
rustup component add rust-src

export AVR_CPU_FREQUENCY_HZ=16000000

cargo build -Z build-std=core --target avr-atmega328p.json --release

lsusb
ls -l /dev/bus/usb

avrdude -patmega328p -carduino -P/dev/ttyUSB0  -b57600 -D -Uflash:w:target/avr-atmega328p/release/aqua.elf:e



-patmega328p is the AVR part number
-carduino is the programmer
-P[PORT] is the serial port of your connected Arduino
    On Linux & macOS, replace [PORT] with your Arduino's serial port (like /dev/ttyUSB0)
-b115200 is the baud rate
-D disables flash auto-erase
-Uflash:w:target/avr-atmega328p/release/blink.elf:e writes the blink.elf program to the Arduino's flash memory


From https://unix.stackexchange.com/questions/144029/command-to-determine-ports-of-a-device-like-dev-ttyusb0:

for sysdevpath in $(find /sys/bus/usb/devices/usb*/ -name dev); do
    (
        syspath="${sysdevpath%/dev}"
        devname="$(udevadm info -q name -p $syspath)"
        [[ "$devname" == "bus/"* ]] && exit
        eval "$(udevadm info -q property --export -p $syspath)"
        [[ -z "$ID_SERIAL" ]] && exit
        echo "/dev/$devname - $ID_SERIAL"
    )
done