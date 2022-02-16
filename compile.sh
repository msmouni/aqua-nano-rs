export AVR_CPU_FREQUENCY_HZ=16000000
cargo build -Z build-std=core --target avr-atmega328p.json --release