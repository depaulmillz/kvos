;this is the first 
;in this link https://wiki.osdev.org/Multiboot#External_Links, multiboot2 explain the format
section .multiboot_header
header_start:
  dd 0xe85250d6 ; magic number
  dd 0          ; boot into i386
  dd header_end - header_start ; length
  dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

  ; multiboot tags

  dw 0
  dw 0
  dd 8

header_end:
