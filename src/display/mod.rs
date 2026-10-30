//! Optional guest display adapter. Presentation never touches device registers.
mod font;
mod glyphs;
use crate::mmu::display_memory as memory;
mod summary;
pub mod view;
use core::{arch::asm, ptr};
pub const WIDTH: usize = 480;
pub const HEIGHT: usize = 800;
const CONFIG: usize = memory::END - 4096;
const DMA: usize = CONFIG + 32;
const _: () = assert!(WIDTH * HEIGHT * 4 <= CONFIG - memory::START);
// SAFETY: linker places this sole reservation at START, outside stacks/page pool.
// No Rust reference is formed to it; the display owner uses bounded volatile access.
#[used]
#[unsafe(link_section = ".framebuffer")]
static mut ARENA: [u8; memory::END - memory::START] = [0; memory::END - memory::START];
pub struct Display {
    _exclusive: (),
}
impl Display {
    /// # Safety
    /// Call once on the fixed virt-9.2 board, one EL1 CPU, IRQs masked, caches off.
    /// MMU must map fw_cfg as Device and the reserved arena as EL1-only RW/NX.
    /// No other owner may access fw_cfg/arena. QEMU is the trusted DMA device.
    pub unsafe fn initialize() -> Result<Option<Self>, &'static str> {
        select(0);
        if read::<4>() != *b"QEMU" {
            return Err("display_bad_fw_cfg");
        }
        select(1);
        if u32::from_le_bytes(read::<4>()) & 2 == 0 {
            return Err("display_no_dma");
        }
        select(0x19);
        let count = u32::from_be_bytes(read::<4>());
        if count > 256 {
            return Err("display_directory_limit");
        }
        let mut selected = None;
        for _ in 0..count {
            let entry = read::<64>();
            if &entry[8..18] == b"etc/ramfb\0" {
                let size = u32::from_be_bytes(entry[0..4].try_into().unwrap());
                let key = u16::from_be_bytes([entry[4], entry[5]]);
                if size != 28 || !(0x20..0x4000).contains(&key) {
                    return Err("display_bad_config");
                }
                selected = Some(key);
            }
        }
        let Some(key) = selected else {
            return Ok(None);
        };
        let mut display = Self { _exclusive: () };
        display.fill(0xf6f7f2);
        let words = [
            0,
            memory::START as u32,
            0x3432_5258,
            0,
            WIDTH as u32,
            HEIGHT as u32,
            (WIDTH * 4) as u32,
        ];
        // SAFETY: descriptor/config live in a dedicated reserved page, not the stack.
        // They remain valid even if DMA fails/times out. All fields are big endian;
        // barriers order non-cacheable RAM writes before MMIO and DMA observations.
        unsafe {
            for (i, word) in words.iter().enumerate() {
                ptr::write_volatile((CONFIG + 4 * i) as *mut u32, word.to_be());
            }
            ptr::write_volatile(DMA as *mut u32, ((u32::from(key) << 16) | 0x18).to_be());
            ptr::write_volatile((DMA + 4) as *mut u32, 28_u32.to_be());
            ptr::write_volatile((DMA + 8) as *mut u64, (CONFIG as u64).to_be());
            asm!("dsb sy", options(nostack, preserves_flags));
            ptr::write_volatile((memory::FW_CFG + 16) as *mut u64, (DMA as u64).to_be());
            for _ in 0..100_000 {
                let control = u32::from_be(ptr::read_volatile(DMA as *const u32));
                if control == 0 {
                    asm!("dsb sy", options(nostack, preserves_flags));
                    return Ok(Some(display));
                }
                if control & 1 != 0 {
                    return Err("display_dma_error");
                }
                core::hint::spin_loop();
            }
        }
        Err("display_dma_timeout")
    }
    pub fn pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < WIDTH && y < HEIGHT {
            // SAFETY: the unique display owner clips every pixel to the framebuffer;
            // its identity mapping is writable/non-executable. QEMU only reads pixels.
            unsafe {
                ptr::write_volatile(
                    (memory::START as *mut u32).add(y * WIDTH + x),
                    color & 0x00ff_ffff,
                );
            }
        }
    }
    pub fn blend(&mut self, x: i32, y: i32, color: u32, alpha: u32) {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            return;
        }
        let offset = y as usize * WIDTH + x as usize;
        // SAFETY: clipped index in exclusively owned, initialized framebuffer RAM.
        let old = unsafe { ptr::read_volatile((memory::START as *const u32).add(offset)) };
        let alpha = alpha.min(255);
        let mut result = 0;
        for shift in [0, 8, 16] {
            let channel =
                (((color >> shift) & 255) * alpha + ((old >> shift) & 255) * (255 - alpha) + 127)
                    / 255;
            result |= channel << shift;
        }
        self.pixel(x as usize, y as usize, result);
    }
    pub fn fill(&mut self, color: u32) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.pixel(x, y, color);
            }
        }
    }
    pub fn present(&self) {
        // SAFETY: EL1 barrier orders completed pixel writes for QEMU's display read.
        unsafe {
            asm!("dsb sy", options(nostack, preserves_flags));
        }
    }
}
fn select(key: u16) {
    // SAFETY: only initialize calls this with exclusive device ownership, aligned
    // Device-nGnRnE MMIO mapped at the fixed board's selector register.
    unsafe {
        ptr::write_volatile((memory::FW_CFG + 8) as *mut u16, key.to_be());
    }
}
fn read<const N: usize>() -> [u8; N] {
    let mut bytes = [0; N];
    for byte in &mut bytes {
        // SAFETY: exclusive fw_cfg stream; byte reads advance the selected item.
        *byte = unsafe { ptr::read_volatile(memory::FW_CFG as *const u8) };
    }
    bytes
}
