global long_mode_start


section .text
bits 64 ; compile for x86_64
long_mode_start:
  
  ; 0 the data segment registers
  mov ax, 0
  mov ss, ax
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  mov rax, 0x2f592f412f4b2f4f
  mov qword [0xb8000], rax ; print OK to the screen 

  extern kernel_main
  call kernel_main

  hlt ; halts the CPU until the next external interrupt is received (in case kernel_main ever returns).
 
