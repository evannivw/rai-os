#![no_std]
#![no_main]

mod clock;
mod command;
mod console;
mod context;
mod exceptions;
mod memory;
mod mmu;
mod protocol;
mod schedule;
mod task;
mod timer;
mod uart;
mod user;

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
    // SAFETY: sole pool construction; linker reserves it after code/data/stacks.
    // Our fixed QEMU machine maps this RAM exclusively, with one CPU and MMU off.
    let mut memory = unsafe { memory::Memory::from_linker() };
    // SAFETY: single EL1 CPU, masked interrupts, cold MMU/caches; linker bounds
    // and the fixed-board mapper cover all live kernel memory plus UART.
    let mmu = unsafe { mmu::Mmu::enable(&mut memory) };
    // SAFETY: sole GIC/timer owner; EL1, masked IRQs, fixed identity MMIO mappings.
    let mut timer = unsafe { timer::Timer::initialize(clock.snapshot().counter_hz) };
    uart::write("RAI-OS: boot OK | AArch64 EL1 | UART PL011\n");
    #[cfg(feature = "exception-self-test")]
    exceptions::self_test();
    #[cfg(any(
        feature = "mmu-test-write-text",
        feature = "mmu-test-write-rodata",
        feature = "mmu-test-write-table",
        feature = "mmu-test-execute-data",
        feature = "mmu-test-unmapped"
    ))]
    mmu::run_fault();
    #[cfg(any(
        feature = "context-test-unknown-svc",
        feature = "context-test-wrong-site"
    ))]
    context::self_test();
    #[cfg(feature = "timer-test-unexpected-irq")]
    {
        uart::write("SELF-TEST: unexpected SGI 1\n");
        timer.check().expect("unexpected IRQ test resumed");
    }
    console::run(&clock, &mut memory, &mmu, &mut timer)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::write("PANIC: kernel halted\n");
    loop {
        core::hint::spin_loop();
    }
}

// Fault-injection builds run one intentional exception at a time.
const _: () = assert!(
    cfg!(feature = "exception-self-test") as usize
        + cfg!(feature = "isolation-test-open-peer") as usize
        + cfg!(feature = "lifecycle-test-spin") as usize
        + cfg!(feature = "lifecycle-test-both-spin") as usize
        + cfg!(feature = "lifecycle-test-invalid-exit") as usize
        + cfg!(feature = "lifecycle-test-bad-call") as usize
        + cfg!(feature = "schedule-test-corrupt-register") as usize
        + cfg!(feature = "schedule-test-user-fault") as usize
        + cfg!(feature = "timer-test-unexpected-irq") as usize
        + cfg!(feature = "user-test-syscall-loop") as usize
        + cfg!(feature = "user-test-mask-irq") as usize
        + cfg!(feature = "user-test-disable-timer") as usize
        + cfg!(feature = "user-test-expired-syscall") as usize
        + cfg!(feature = "user-test-invalid-calls") as usize
        + cfg!(feature = "user-test-read-kernel") as usize
        + cfg!(feature = "user-test-write-code") as usize
        + cfg!(feature = "user-test-uart") as usize
        + cfg!(feature = "user-test-gic") as usize
        + cfg!(feature = "user-test-execute-stack") as usize
        + cfg!(feature = "user-test-privileged") as usize
        + cfg!(feature = "context-test-unknown-svc") as usize
        + cfg!(feature = "context-test-wrong-site") as usize
        + cfg!(feature = "mmu-test-write-text") as usize
        + cfg!(feature = "mmu-test-write-rodata") as usize
        + cfg!(feature = "mmu-test-write-table") as usize
        + cfg!(feature = "mmu-test-execute-data") as usize
        + cfg!(feature = "mmu-test-unmapped") as usize
        <= 1,
    "Select at most one fault-injection feature"
);
