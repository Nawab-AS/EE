# Extended Essay

The source code for my IBDP computer science EE.
Refer to EE for methodology behind code.


## Requirements
- Rust (`1.94.0` was used)
- Python
- Python `pyserial` library
- `picotool` cli
- Raspberry Pi Pico 2 (RP2350)


## How to use

### Part 1: Cloning this repo
```bash
cd ~
git clone https://github.com/nawab-as/ee.git
```


### Part 2: Generating the prime lookup table
The generated lookup table will be a relatively large file (~220kb, although very compressible) and hence is not included in this repo.
Additionally, this should be run on a separate computer than the Pico as the generated prime numbers are seeded and will not be manipulated by CPU architecture or OS, and also due to the fact that the Pico doesn't have any built-in filesystem drivers.

1) Navigate to the `~/ee/primegen` directory
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


### Part 3: Compiling and flashing to the Pico
1) Navigate to the `~/ee/ee-experiment` directory
```bash
cd ~/ee/ee-experiment
```

2) Install the cross-compile toolchain
```bash
rustup target add thumbv8m.main-none-eabihf
```

3) Plug in the Pico in BOOTSEL via USB

4) Compile and run the Rust project

The release build is configured to use `picotool` to flash and run automatically after compilation.
```bash
cargo run --release
```

### Part 4: Recording data
1) Navigate to the `~/ee/data-receiver` directory
```bash
cd ~/ee/data-receiver
```

2) Connect the Pico via USB normally

3) Run the data receiver code

This will automatically initialize the serial connection and, when finished, will parse all data into a `.csv` file.
```bash
python ./main.py
```


