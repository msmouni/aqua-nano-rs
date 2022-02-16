 http://arahna.de/arduino-command-line/
 
 Communication with Arduino from Linux-Terminal

I already wrote how to send the message from/to Arduino with help Python. Now I want tell you, how to send/receive the messages from/to Arduino in Linux-Terminal.

As you know, all devices of serial ports are represented by device files in the /dev directory. It is through these files that Linux OS communicates with an external device on the serial port. To transfer something to an external device, you need to write data to this file, and to read information from the device, read data from the file. This can be done with the cat and echo commands, as for "usual" disk files.

How to work with a COM port from the command line? Three commands are used for this: stty, cat and echo.

The stty command sets the parameters and speed of the COM port. Its format is:

stty [-F DEVICE | --file=DEVICE] [SETTING]...

for our case:

$ stty 9600 -F /dev/ttyUSB0 raw -echo

The raw parameter establishes that the data is transferred to the computer byte-by-byte the same way, as they arrive at the port without changes (more see man stty).

Code Arduino:

void setup(){

  Serial.begin(9600);

}

 

void loop(){

   int bytesSent = Serial.println("hello from Arduino");

   delay(5000);

}

Enter the command in the console:

$ cat /dev/ttyUSB0

we can see:

To see the hex data codes coming from the device, use the hexdump command.

$ cat /dev/ttyUSB0|hexdump -C

...