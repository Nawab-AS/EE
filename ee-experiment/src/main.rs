#![no_std]
#![no_main]

// heap setup
extern crate alloc;
use core::usize;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 16 * 1024; // kb
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

fn init_heap() {
    unsafe {
        let heap_ptr = &raw mut HEAP as *mut _ as *mut u8;
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}


use panic_halt as _;
use rp235x_hal as hal;
use cortex_m_rt;
use cortex_m::{peripheral::Peripherals, asm};

mod logger;
mod lookup;
mod helper;
mod ecc;
mod rsa;

// consts
static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None;
const XTAL_FREQ_HZ: u32 = 12_000_000;
const TRIALS_PER_KEY: usize = 100;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[cortex_m_rt::entry]
fn main() -> ! {
    // setup
    init_heap();
    let mut pac = hal::pac::Peripherals::take().unwrap(); // peripheral access
    let mut cp = Peripherals::take().unwrap(); // cortex-m peripherals
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG); // needed for clocks

    // Initialize DWT cycle counter
    cp.DCB.enable_trace();
    unsafe {
        cp.DWT.cyccnt.write(0); // Reset cycle counter
    }
    cp.DWT.enable_cycle_counter();

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // Initialize USB serial
    let usb_bus = hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );
    unsafe {
        USB_BUS = Some(usb_device::bus::UsbBusAllocator::new(usb_bus));
        let usb_bus_ref = core::ptr::addr_of!(USB_BUS)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        logger::init_usb_serial(usb_bus_ref);
    }

    // Wait for "START" from serial
    let mut buf = [0u8; 16];
    loop {
        logger::poll_usb();
        if let Some(len) = logger::read_line(&mut buf) {
            if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                if s == "START" {
                    break;
                }
            }
        }
    }
    
    uprint!("=== Started EE Experiment ===\n");
    

    // MAIN PROGRAM
    const TOTAL_SIZES: u16 = ((lookup::PRIME_RANGE[1] - lookup::PRIME_RANGE[0]) / 8 + 1) as u16;
    let mut results = [(0u32, 0u32, 0u16, 0u16); lookup::TRIALS as usize]; // (ECC time, RSA time, ECC fails, RSA fails)
    for bits_ in 0..TOTAL_SIZES {
        let bits = bits_ * 8 + lookup::PRIME_RANGE[0];
        uprint!("Processing: {} bits\n", bits);
        for i in 0..(lookup::TRIALS as u16) {
            uprint!("  Trial #{}...\n", i + 1);
            let mut ecc_time: u64 = 0;
            let mut rsa_time: u64 = 0;
            
            let trial_data: lookup::KeySize = lookup::LOOKUP_TABLE[(bits * (lookup::TRIALS as u16) + i) as usize];
            for _ in 0..TRIALS_PER_KEY {
                // ECC
                let mut start = cp.DWT.cyccnt.read();
                if !ecc::ECDH(trial_data.ecc) {
                    results[i as usize].2 += 1;
                    uprint!("[ERROR] ECC key exchange failed");
                }
                let mut end = cp.DWT.cyccnt.read();
                ecc_time += end.wrapping_sub(start) as u64;


                // RSA
                start = cp.DWT.cyccnt.read();
                if !rsa::KEY_TRANSPORT(trial_data.rsa) {
                    results[i as usize].3 += 1;
                    uprint!("[ERROR] RSA key transport failed");
                }
                end = cp.DWT.cyccnt.read();
                rsa_time += end.wrapping_sub(start) as u64;
            }
            results[i as usize] = ((ecc_time / (TRIALS_PER_KEY as u64)) as u32, (rsa_time / (TRIALS_PER_KEY as u64)) as u32, results[i as usize].2, results[i as usize].3);
        }
        
        // Send results for this key size
        uprint!("=== Results for {} bits ===\n", bits);
        
        for i in 0..(results.len()) {
            uprint!("Trial #{}: ECC = {}, RSA = {}, ECC fails = {}, RSA fails = {}\n", i % (lookup::TRIALS as usize) + 1, results[i].0, results[i].1, results[i].2, results[i].3);
        }
        uprint!("\n\n");

        results = [(0u32, 0u32, 0u16, 0u16); lookup::TRIALS as usize]; // reset for next key size
    }

    uprint!("=== Experiment Complete ===\n");
    helper::exit()
}
