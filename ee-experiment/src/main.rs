#![no_std]
#![no_main]
extern crate alloc;
use alloc::format;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Heap size (adjust as needed)
const HEAP_SIZE: usize = 16 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

fn init_heap() {
    unsafe {
        let heap_ptr = &raw mut HEAP as *mut _ as *mut u8;
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}

mod logger;

use panic_halt as _;
use rp235x_hal as hal;
use cortex_m_rt;

// consts
static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None;
const XTAL_FREQ_HZ: u32 = 12_000_000;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

// Calculate the nth fibonacci number
fn fibonacci(n: u8) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    
    for n1 in 2..=n {
        let temp = a.saturating_add(b);
        a = b;
        b = temp;
        let line = format!("n = {}, a = {}, b = {}\r\n", n1, a, b);
        uprint!("{}", line);
    }
    
    b
}

// Parse input string to a number, trimming whitespace
fn parse_input(buf: &[u8]) -> Result<u8, &'static str> {
    let s = core::str::from_utf8(buf).map_err(|_| "Invalid UTF-8")?;
    let trimmed = s.trim();
    
    if trimmed == "STOP" {
        return Err("STOP");
    }
    
    let num: u8 = trimmed.parse().map_err(|_| "Invalid number")?;
    Ok(num)
}

#[cortex_m_rt::entry]
fn main() -> ! {
    // setup
    init_heap();
    let mut pac = hal::pac::Peripherals::take().unwrap(); // peripheral access
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG); // needed for clocks

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

    // 1. STARTUP - Wait for "START" from serial
    
    let mut buf = [0u8; 16];
    loop {
        logger::poll_usb();
        if let Some(len) = logger::read_line(&mut buf) {
            if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                if s.trim() == "START" {
                    break;
                }
            }
        }
    }
    
    // Display startup message
    uprint!("=== Fibonacci Calculator Started ===\r\n");
    uprint!("Enter a number (0-93) to calculate fibonacci\r\n");
    uprint!("Enter 'STOP' to exit\r\n");
    
    // 2. MAIN LOOP
    let mut input_buf = [0u8; 32];
    loop {
        logger::poll_usb();
        
        // Ask for input
        uprint!("Generate the nth fibonacci number: n = ");
        
        // Wait for input
        loop {
            logger::poll_usb();
            if let Some(len) = logger::read_line(&mut input_buf) {
                // Parse the input
                match parse_input(&input_buf[..len]) {
                    Ok(n) => {
                        // Calculate and display fibonacci number
                        let fib_n = fibonacci(n);
                        let line = format!("fibonacci({}) = {}\r\n", n, fib_n);
                        uprint!("{}", line);
                        break; // Break inner loop, continue main loop
                    }
                    Err("STOP") => {
                        // 3. EXIT
                        uprint!("Stopping\r\n");
                        loop {} // Infinite loop to "exit"
                    }
                    Err(e) => {
                        uprint!("Error: {}. Please enter a valid number (0-93) or 'STOP'\r\n", e);
                        uprint!("Generate the nth fibonacci number: n = ");
                    }
                }
            }
        }
    }
}
