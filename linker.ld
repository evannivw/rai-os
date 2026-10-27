ENTRY(_start)
MEMORY { RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 127M }
SECTIONS {
    .text : { KEEP(*(.text.boot)) KEEP(*(.text.vectors)) *(.text .text.*) } > RAM
    .rodata : ALIGN(16) { *(.rodata .rodata.*) } > RAM
    .data : ALIGN(16) { *(.data .data.*) } > RAM
    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*) *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    } > RAM
    .stack (NOLOAD) : ALIGN(16) {
        __stack_bottom = .;
        . += 64K;
        __stack_top = .;
    } > RAM
    .exception_stack (NOLOAD) : ALIGN(16) {
        __exception_stack_bottom = .;
        . += 16K;
        __exception_stack_top = .;
    } > RAM
    __kernel_reserved_end = .;
    .page_pool (NOLOAD) : ALIGN(4096) {
        __page_pool_start = .;
        . += 1M;
        __page_pool_end = .;
    } > RAM
    ASSERT(__page_pool_start >= __kernel_reserved_end, "Page pool overlaps kernel")
    ASSERT((__page_pool_start & 4095) == 0, "Page pool must be aligned")
    ASSERT(__page_pool_end <= 0x48000000, "Page pool exceeds fixed QEMU RAM")
    ASSERT((exception_vectors & 2047) == 0, "EL1 vectors must be 2 KiB aligned")
    ASSERT(exception_vectors_end - exception_vectors == 2048, "EL1 vectors must contain 16 slots")
    /DISCARD/ : { *(.eh_frame*) }
}
