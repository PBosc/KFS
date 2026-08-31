BITS 32

global start
extern kernel_main

SECTION .text

start:
    call kernel_main
.hang:
    hlt
    jmp .hang
