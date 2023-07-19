# -e: Exit immediately if a pipeline returns a non-zero status
# -x: Print a trace of simple commands
set -ex

echo -e "\n______________________________________"
echo "Rust Version:"
rustup show active-toolchain -v
echo -e "______________________________________\n"

export AVR_CPU_FREQUENCY_HZ=16000000
cargo clippy -Z build-std=core --target avr-atmega328p.json --release -- -D warnings
cargo build -Z build-std=core --target avr-atmega328p.json --release