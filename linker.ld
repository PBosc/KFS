ENTRY(start)

SECTIONS {
    .boot 1M : {
        KEEP(*(.multiboot_header))
    }

    .text : {
        *(.text .text.*)
    }
}