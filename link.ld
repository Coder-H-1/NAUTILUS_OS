ENTRY(_start)
SECTIONS
{
    . = 0x80000;
    .text : { KEEP(*(.text._start)) *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss : {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss*)
        __bss_end = .;
    }
}
