use core::cell::RefCell;
use core::fmt;
use cortex_m::interrupt::Mutex;
use rp235x_hal::usb::UsbBus;
use usb_device::prelude::*;
use usbd_serial::SerialPort;
const NAME: &str = "EE Experiment";

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
            .serial_number("00000000")])
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

// Reads a line from USB serial into the provided buffer. Returns Some(len) if a line is read, None otherwise.
pub fn read_line(buf: &mut [u8]) -> Option<usize> {
    use cortex_m::interrupt;
    let mut len = 0;
    let mut found_newline = false;
    interrupt::free(|cs| {
        if let Some(ref mut usb) = *USB_SERIAL.borrow(cs).borrow_mut() {
            let mut tmp = [0u8; 64];
            if let Ok(count) = usb.serial.read(&mut tmp) {
                for &b in &tmp[..count] {
                    if len < buf.len() {
                        buf[len] = b;
                        len += 1;
                        if b == b'\n' || b == b'\r' {
                            found_newline = true;
                            break;
                        }
                    }
                }
            }
        }
    });
    if len > 0 && found_newline {
        Some(len)
    } else {
        None
    }
}

#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        cortex_m::interrupt::free(|cs| {
            if let Some(ref mut usb) = *$crate::logger::USB_SERIAL.borrow(cs).borrow_mut() {
                let _ = core::write!(usb, $($arg)*);
                usb.poll();
                // small delay to avoid race condition
                for _ in 0..10_000 {
                    cortex_m::asm::nop();
                }
            }
        });
    }};
}