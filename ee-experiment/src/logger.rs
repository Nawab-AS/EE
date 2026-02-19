use core::cell::RefCell;
use core::fmt;
use cortex_m::interrupt::Mutex;
use rp235x_hal::usb::UsbBus;
use usb_device::prelude::*;
use usbd_serial::SerialPort;
const NAME: &str = "EE Experiment";

// Input buffer for reading serial data
static mut INPUT_BUFFER: [u8; 256] = [0; 256];
static mut INPUT_POS: usize = 0;

pub static USB_SERIAL: Mutex<RefCell<Option<UsbSerial>>> = Mutex::new(RefCell::new(None));

pub struct UsbSerial {
    pub serial: SerialPort<'static, UsbBus>,
    pub usb_dev: UsbDevice<'static, UsbBus>,
}

impl UsbSerial {
    pub fn poll(&mut self) {
        self.usb_dev.poll(&mut [&mut self.serial]);
    }
}

impl fmt::Write for UsbSerial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write(s.as_bytes()).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

pub fn init_usb_serial(usb_bus: &'static usb_device::bus::UsbBusAllocator<UsbBus>) {
    let serial = SerialPort::new(usb_bus);

    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Raspberry Pi")
            .product(NAME)
            .serial_number("12345")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    cortex_m::interrupt::free(|cs| {
        USB_SERIAL
            .borrow(cs)
            .replace(Some(UsbSerial { serial, usb_dev }));
    });
}

pub fn poll_usb() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut usb) = *USB_SERIAL.borrow(cs).borrow_mut() {
            usb.poll();
        }
    });
}

// Reads input from USB serial into the provided buffer. Returns Some(len) if data is available, None otherwise.
pub fn read_line(buf: &mut [u8]) -> Option<usize> {
    use cortex_m::interrupt;
    
    // try to read more data from USB and add to the buffer
    interrupt::free(|cs| {
        if let Some(ref mut usb) = *USB_SERIAL.borrow(cs).borrow_mut() {
            let mut tmp = [0u8; 64];
            if let Ok(count) = usb.serial.read(&mut tmp) {
                unsafe {
                    let input_pos_ptr = core::ptr::addr_of_mut!(INPUT_POS);
                    let input_buffer_ptr = core::ptr::addr_of_mut!(INPUT_BUFFER);
                    
                    for i in 0..count {
                        if *input_pos_ptr < (*input_buffer_ptr).len() {
                            (*input_buffer_ptr)[*input_pos_ptr] = tmp[i];
                            *input_pos_ptr += 1;
                        }
                    }
                }
            }
        }
    });
    
    // Return whatever data is in the buffer
    unsafe {
        let input_pos_ptr = core::ptr::addr_of_mut!(INPUT_POS);
        let input_buffer_ptr = core::ptr::addr_of_mut!(INPUT_BUFFER);
        
        if *input_pos_ptr > 0 {
            let data_len = (*input_pos_ptr).min(buf.len());
            // Copy data to output buffer
            core::ptr::copy((*input_buffer_ptr).as_ptr(), buf.as_mut_ptr(), data_len);
            
            // Clear the input buffer
            *input_pos_ptr = 0;
            
            return Some(data_len);
        }
    }
    
    None
}

#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut usb) = *$crate::logger::USB_SERIAL.borrow(cs).borrow_mut() {
                let _ = core::write!(usb, $($arg)*);
                asm::delay(1_000);

                usb.poll();

                asm::delay(1_000);
            }
        });
    }};
}