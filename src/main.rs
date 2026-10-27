#![no_std]
#![no_main]

mod clock;
mod command;
mod console;
mod protocol;
mod task;
mod uart;

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.S"));

// SAFETY: the unique symbol is called only by boot.S after stack/BSS setup.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let clock = clock::Clock::start();
    uart::init();
    let Some(clock) = clock else {
        panic!("invalid counter frequency");
    };
    uart::write("RAI-OS: boot OK | AArch64 EL1 | UART PL011\n");
    console::run(&clock)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::write("PANIC: kernel halted\n");
    loop {
        core::hint::spin_loop();
    }
}
