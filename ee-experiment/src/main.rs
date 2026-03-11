#![no_std]
#![no_main]

// heap
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
mod ecc;
mod rsa;

// consts
static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None;
const XTAL_FREQ_HZ: u32 = 12_000_000;
const TRIALS_PER_KEY: usize = 15;

pub fn exit() -> ! {
    uprint!("Exiting...\n");
    loop {}
}

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

    // init DWT cycle counter
    cp.DCB.enable_trace();
    unsafe {
        cp.DWT.cyccnt.write(0);
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

    // init USB serial
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

    // wait for "START" over serial
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

    for size_idx in 0..lookup::ECC_V_RSA.len() {
        let (ecc_bits, rsa_bits) = lookup::ECC_V_RSA[size_idx];
        uprint!("=== ECC {} / RSA {} bits ===\n", ecc_bits, rsa_bits);

        for i in 0..(lookup::TRIALS as usize) {
            let mut ecc_time: u64 = 0;
            let mut rsa_time: u64 = 0;
            let mut ecc_fails: u16 = 0;
            let mut rsa_fails: u16 = 0;
            
            let trial_data: lookup::KeySize = lookup::LOOKUP_TABLE[size_idx * (lookup::TRIALS as usize) + i];

            let ecc_ctx = ecc::EccCtx::new(trial_data.ecc.curve.p, trial_data.ecc.curve.a);
            let rsa_ctx = rsa::RsaCtx::new(&trial_data.rsa);

            for _j in 0..TRIALS_PER_KEY {
                // ECC
                let mut start = cp.DWT.cyccnt.read();
                if !ecc::ecdh(trial_data.ecc, &ecc_ctx) {
                    ecc_fails += 1;
                    uprint!("[ERROR] ECC key exchange failed");
                }
                let mut end = cp.DWT.cyccnt.read();
                ecc_time += end.wrapping_sub(start) as u64;
                logger::poll_usb();

                // RSA
                start = cp.DWT.cyccnt.read();
                if !rsa::key_transport(trial_data.rsa, &rsa_ctx) {
                    rsa_fails += 1;
                    uprint!("[ERROR] RSA key transport failed");
                }
                end = cp.DWT.cyccnt.read();
                rsa_time += end.wrapping_sub(start) as u64;
                logger::poll_usb();
            }

            let avg_ecc = (ecc_time / (TRIALS_PER_KEY as u64)) as u32;
            let avg_rsa = (rsa_time / (TRIALS_PER_KEY as u64)) as u32;
            uprint!("Trial #{}: ECC = {}, RSA = {}, ECC fails = {}, RSA fails = {}\n", i + 1, avg_ecc, avg_rsa, ecc_fails, rsa_fails);
        }
        uprint!("\n");
    }

    uprint!("=== Experiment Complete ===\n");
    exit()
}
