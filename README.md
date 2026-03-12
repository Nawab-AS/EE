# Extended Essay

The source code for my IBDP computer science EE.
Refer to EE for methodology behind code.


## Requirements
- Rust (`1.94.0` was used)
- Python
- Python `pyserial` library
- `picotool` cli
- Raspberry pi pico 2 (rp2350)


## How to use

### Part 1: Cloning this repo
```bash
cd ~
git clone https://github.com/nawab-as/ee.git
```


### Part 2: Generating the prime lookup table
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
1) Navigate to the `/ee-experiment` directory
```bash
cd ~/ee/ee-experiment
```

2) Install the cross-compile toolchain
```bash
rustup target add thumbv8m.main-none-eabihf
```

3) Plug in the pico in BOOTSEL via USB

4) Compile and run the rust project
The release build is configured to use `picotool` to flash and run automatically after it is compiled.
```bash
cargo run --release
```

### Part 3: Recording data
1) Navigate to the `~/data-receiver` directory
```bash
cd ~/ee/ee-experiment
```

1) Connect the pico via USB normally

2) Run the data receiver code
This will automatically initialize the serial connection and when finished, will parse all data into a `.csv` file.
```bash
python ./main.py
```
