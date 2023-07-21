stty 115200 -F /dev/ttyUSB0 raw -echo

cat /dev/ttyUSB0

# To see the hex data codes coming from the device, use the hexdump command.
# cat /dev/ttyUSB0|hexdump -C