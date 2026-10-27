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
    ASSERT((exception_vectors & 2047) == 0, "EL1 vectors must be 2 KiB aligned")
    ASSERT(exception_vectors_end - exception_vectors == 2048, "EL1 vectors must contain 16 slots")
    /DISCARD/ : { *(.eh_frame*) }
}
