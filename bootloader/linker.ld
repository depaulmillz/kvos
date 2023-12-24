ENTRY(start) /* the entry pointer is label start in boot.asm */

SECTIONS {
  . = 1M; /* The following sections will start being placed in the resulting binary at an offset of 1MB. All on a 4K boundary. */

  .rodata : /* read-only data */
  {
    KEEP(*(.multiboot_header)) /* prevent optimizing the header out */
    *(.rodata .rodata.*) /* includes all .rodata sections from input files */
    . = ALIGN(4K);
  }

  .text : ALIGN(4K) 
  {
    *(.text .text.*) /* All .text sections from input files are included. */ 
    . = ALIGN(4K);
  }

  .data : { /* Holds initialized data and ensures that it starts on a 4KB boundary. */
    *(.data .data.*) 
    . = ALIGN(4K);
  }

  .bss : 
  {
    *(.bss .bss.*)
    . = ALIGN(4K);
  }

  .got : 
  {
    *(.got)
    . = ALIGN(4K);
  }

  .got.plt : 
  {
    *(.got.plt)
    . = ALIGN(4K);
  }

  .data.rel.ro : ALIGN(4K) { /* Contains data that is read-only during execution but can be modified during program startup */
    *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    . = ALIGN(4K);
  }

  .gcc_except_table : ALIGN(4K) { /* Contains data used for exception handling. */
    *(.gcc_except_table)
    . = ALIGN(4K);
  }

  .eh_frame : ALIGN(4K) { /* Contains data used for unwinding the stack. */
    *(.eh_frame)
  }

}
