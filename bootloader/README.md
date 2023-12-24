# overview of bootloader 
Here is a brief summary of how the bootloader works:

The bootloader implementation is split into multiple assembly source files that get compiled and linked to create a bootable kernel image.

1. multiboot_header.asm contains the multiboot header needed by GRUB to load the kernel. This defines the entry point.

2. boot.asm does initial setup like entering protected mode and enabling paging. It gets the system ready for long mode.

3. long_mode_init.asm finalizes the transition to long mode by zeroing registers and jumping to kernel.

4. The object files are linked together using linker.ld which defines the layout of the compiled ELF file.

5. grub.cfg and grub-test.cfg configure GRUB to load the kernel. grub.cfg loads the kernel without any arguments by default.
grub-test.cfg loads the kernel with the test argument by default.

The key steps are:
1. Compile asm to objects
2. Link objects with the kernel static library 
3. Make ISO with GRUB, kernel, configs
4. Boot ISO in QEMU

---
Details of each file
---

## src/
bootloader implementation is seperate into following files.

### src/multiboot_header.asm
The multiboot header needs to be at the very beginning of the bootloader binary. 

1. BIOS loads GRUB bootloader

2. GRUB loads the kernel file into memory 

3. GRUB searches for the multiboot header at the beginning of the kernel

4. GRUB validates the magic number, checksum, etc in the header 

5. If valid, GRUB jumps to the address specified in the header

6. This transfers control to the start label defined in the multiboot header

7. The bootloader assembly code takes over from there

The key responsibilities of the multiboot_header.asm are:

- Provide the magic number, checksum, etc for GRUB to validate

- Define the entry point address for GRUB to jump to

- Mark the binary as multiboot2 compliant 

This is what allows GRUB to successfully load and transfer control to the bootloader code. Without the header, GRUB would not know how to boot the custom kernel file.

### src/boot.asm
After the multiboot header, the next key component is boot.asm. 

1. Sets up the stack pointer and other registers

2. Performs multiboot validation checks

3. Tests for CPU features like long mode

4. Enables protected mode

5. Sets up paging by mapping page tables  

6. Enables long mode

7. Jumps to the long mode initialization code

Some key responsibilities of boot.asm:

- Validation of multiboot loading
- Protected mode and paging setup
- Testing CPU capabilities 
- Entering long mode
- Bridging between the multiboot header and long mode init

So in summary, boot.asm does crucial setup to get the system into long mode with paging enabled. It provides the foundation for the kernel to run in 64-bit mode. Without boot.asm, the kernel would not have the proper environment to execute.

### src/long_mode_init.asm
long_mode_init.asm  does the final steps once paging and long mode are active.

1. Zero out the segment registers - This is required in 64-bit long mode. The segment registers must be cleared.

2. Print a visual indicator - Writes OK to VGA to visually confirm long mode is active. 

3. Jump to the kernel - Calls the kernel_main() function to transfer control to the higher level kernel code.

Some key points:

- At this point paging and long mode are fully set up by boot.asm.

- The segment registers are invalid in long mode and must be zeroed.

- A visual indicator is helpful to confirm the bootloader entered long mode.

- Jumping to the kernel C code completes the bootloader hand-off. 

So in summary, long_mode_init.asm does the final steps to transition into the kernel by zeroing segment registers, printing an indicator, and jumping to the C entry point. This completes the bootloader's work.

## linker.ld
The linker script is responsible for linking together the separate bootloader object files into the final bootable kernel binary.

Here's a high level overview of what the linker script does:

1. Defines the entry point as the start label from multiboot_header.asm. This is the first code executed.

2. Lays out the sections for .text, .data, .bss, etc. 

3. Ensures proper alignment of sections.

4. Pulls in the object files for each section.

5. Keeps the multiboot header first in the .rodata section.

6. Merges all object files into assigned sections.

7. Resolves any external symbol references between objects.

8. Performs relocation of position-dependent code.

9. Generates the final linked kernel binary.

The linker script ties together the objects files generated from each assembly source into a single executable kernel image that can be booted.

It ensures the multiboot header is at the beginning, aligns sections properly, resolves symbols, and sets the entry point.

So in summary, the linker and linker script are critical to combine the bootloader components into a contiguous, bootable kernel binary in the correct format.

## grub and grub-test
grub.cfg and grub-test.cfg are configuration files that tell GRUB how to load and boot the kernel.

These allow flexible control over the kernel boot options. GRUB will load whichever config file is copied to /boot/grub/grub.cfg at boot time.

The key purpose of these configs is to tell GRUB:

- Which kernel file to load 
- Any arguments to pass to the kernel
- The boot menu options and default entry

This enables booting the custom kernel build with different parameters for production vs testing.
### grub.cfg
It is the main configuration file and boot the kernel normally . It does two key things:

1. Defines a menuentry for "kvos" that loads the multiboot kernel at /boot/kernel.bin

2. Sets the timeout to 0 and default to "kvos" for automatic booting

### grub-test.cfg 
It is a secondary config for testing purposes, config to load kernel in test mode

1. Defines a "kvos-test" entry that passes the "test" argument to kernel.bin 

2. Sets timeout and default to boot to "kvos-test"
