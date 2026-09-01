BITS 32

global start
extern kernel_main
extern __bss_start
extern __bss_end

SECTION .bss
align 16
stack_bottom:
    resb 65536          ; 64 KiB kernel stack
stack_top:

SECTION .text
start:
    ; multiboot2 leaves esp undefined, we set up our own stack first
    mov esp, stack_top

    ; .bss is not guaranteed zeroed by the bootloader; zero it so
    ; lazy_static / spin state start from a known-good value.
    cld
    mov edi, __bss_start
    mov ecx, __bss_end
    sub ecx, edi        ; ecx = byte count
    xor eax, eax
    rep stosb

    ; extern "C", never returns
    call kernel_main

.hang:
    cli
    hlt
    jmp .hang