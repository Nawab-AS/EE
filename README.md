# Extended Essay

The source code for my IBDP computer science EE.
Refer to EE for methodology behind code.


## Requirements
- Rust
- `picotool` cli
- Raspberry pi pico 2 (rp2350)
- Serial input system


## How to use

### Part 1: Cloning this repo
```bash
cd ~
git clone https://github.com/nawab-as/ee.git
```


### Part 2: Generating the prime lookup table
Run using a stable version of rust rustc (I used `1.91.1`).
The generated lookup table will be a relativiely large file (~220kb, although very compressible) and hence is not included in this repo.
Additionally, this should be run on a seperate computer than the pico as the generated prime numbers are seeded and will not be manipulated by cpu architecture or OS and also due to the fact that the pico doesn't have and built-in filesystem drivers.

1) Navigate to the `/primegen` directory
```bash
cd ~/ee/primegen
```

2) Generate the prime lookup table
```bash
cargo run --release
```

3) Copy the generated lookup table to the main experiment
```bash
cp ~/ee/primegen/lookup.rs ~/ee/ee-experiment/src/lookup.rs
```


### Part 3: Compiling and flashing to the pico
This was run with a nightly build of rust. I used `1.94.0-nightly (fa5eda19b 2025-12-12)`.

1) Navigate to the `/ee-experiment` directory
```bash
cd ~/ee/ee-experiment
```

2) Install the cross-compile toolchain
```bash
rustup target add riscv32imac-unknown-none-elf
```

3) Plug in the pico in BOOTSEL via usb

4) Compile and run the rust project
The release build is configured to use `picotool` to flash and run automatically after it is compiled.
```bash
cargo run --release
```

### Part 3: Recording data
This step can be done in various ways. I used the vscode `Serial Moniter` extension.

1) Connect the pico via usb

2) Connect to the pico's serial port
Use a baud rate of 115200

3) Start the experiment
Sent `START` as plain text to the pico via serial. This will notify the pico to continue the execution of the main program past initialization.
After this, the pico will output data via serial.
