ENTRY(_start)
MEMORY { RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 127M }
SECTIONS {
    .text : {
        __text_start = .;
        KEEP(*(.text.boot)) KEEP(*(.text.vectors)) *(.text .text.*)
        . = ALIGN(4096);
        __text_end = .;
    } > RAM
    .rodata : ALIGN(4096) {
        *(.rodata .rodata.*)
        . = ALIGN(4096);
        __rodata_end = .;
    } > RAM
    .data : ALIGN(4096) { __data_start = .; *(.data .data.*) } > RAM
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
    .user_text : ALIGN(4096) {
        __user_text_start = .;
        KEEP(*(.user.text))
        . = ALIGN(4096);
        __user_text_end = .;
    } > RAM
    .user_rodata : ALIGN(4096) {
        KEEP(*(.user.rodata))
        . = ALIGN(4096);
        __user_rodata_end = .;
    } > RAM
    .user_stack (NOLOAD) : ALIGN(4096) {
        . += 4096; // Unmapped lower guard page.
        __user_stack_bottom = .;
        . += 8K;
        __user_stack_top = .;
        . += 4096; // Unmapped upper guard page.
    } > RAM
    __kernel_reserved_end = .;
    .page_pool (NOLOAD) : ALIGN(4096) {
        __page_pool_start = .;
        . += 1M;
        __page_pool_end = .;
    } > RAM
    ASSERT(__page_pool_start >= __kernel_reserved_end, "Page pool overlaps kernel")
    ASSERT((__page_pool_start & 4095) == 0, "Page pool must be aligned")
    ASSERT(__page_pool_end <= 0x40200000, "Kernel/pool exceeds initial MMU RAM window")
    ASSERT(__user_text_end > __user_text_start, "Missing user text")
    ASSERT(__user_rodata_end > __user_text_end, "Missing user constants")
    ASSERT(__user_text_start >= __exception_stack_top, "User overlaps kernel stack")
    ASSERT(__data_start == __rodata_end, "MMU writable boundary mismatch")
    ASSERT((exception_vectors & 2047) == 0, "EL1 vectors must be 2 KiB aligned")
    ASSERT(exception_vectors_end - exception_vectors == 2048, "EL1 vectors must contain 16 slots")
    /DISCARD/ : { *(.eh_frame*) }
}
