// Fixed QEMU virt PL011 mapping; not a portable board driver.
const BASE: usize = 0x0900_0000;
const DR: usize = 0x00;
const FR: usize = 0x18;

fn read_register(offset: usize) -> u32 {
    // SAFETY: all callers use aligned PL011 registers on the supported machine.
    // MMU is off; the device is mapped at BASE. Volatile preserves MMIO reads.
    unsafe { core::ptr::read_volatile((BASE + offset) as *const u32) }
}

fn write_register(offset: usize, value: u32) {
    // SAFETY: same mapping as above; one CPU, interrupts masked. A fatal exception
    // may take over UART, but never resumes the interrupted owner. No Rust
    // references or locks span MMIO operations.
    // Register accesses are 32-bit aligned and volatile; no Rust references alias MMIO.
    unsafe { core::ptr::write_volatile((BASE + offset) as *mut u32, value) }
}

pub fn init() {
    write_register(0x30, 0); // Disable while configuring.
    write_register(0x38, 0); // Mask interrupts; this driver polls.
    write_register(0x44, 0x7ff); // Clear pending interrupts.
    write_register(0x24, 13); // 24 MHz / (16 * 115200).
    write_register(0x28, 1);
    write_register(0x2c, 3 << 5); // Eight bits, no parity, FIFO disabled.
    write_register(0x30, 0x301); // UART + transmit + receive.
}

pub fn put(byte: u8) {
    while read_register(FR) & (1 << 5) != 0 {
        core::hint::spin_loop();
    }
    write_register(DR, u32::from(byte));
}

pub fn write(text: &str) {
    for byte in text.bytes() {
        if byte == b'\n' {
            put(b'\r');
        }
        put(byte);
    }
}

/// Read at most one byte without waiting; clear and report receive errors.
pub fn try_get() -> Result<Option<u8>, ()> {
    if read_register(FR) & (1 << 4) != 0 {
        return Ok(None);
    }
    let data = read_register(DR);
    if data & 0xf00 != 0 {
        write_register(0x04, 0xff);
        Err(())
    } else {
        Ok(Some(data as u8))
    }
}

/// Stream formatted output directly to UART without a heap or temporary string.
pub fn write_display(value: impl core::fmt::Display) {
    use core::fmt::Write;
    struct Output;
    impl Write for Output {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            write(text);
            Ok(())
        }
    }
    // Our Reply formatter and UART sink are infallible; UART still polls and
    // can block if the emulated device stops accepting bytes.
    write!(Output, "{value}").expect("UART formatting failed");
}
