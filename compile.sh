rustup component add rust-src --toolchain nightly-2022-07-10-x86_64-unknown-linux-gnu

export AVR_CPU_FREQUENCY_HZ=16000000
cargo build -Z build-std=core --target avr-atmega328p.json --release