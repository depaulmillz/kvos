; this link contains info about FLAGS in detail: https://en.wikipedia.org/wiki/FLAGS_register which we will use in the code
; this link contains info about cpuid in detail: https://en.wikipedia.org/wiki/CPUID
global start
extern long_mode_start

section .text
bits 32
start:
  mov esp, stack_top
  mov edi, ebx ; grub pushed pointer to multiboot_header struct to ebx
  call check_multiboot
  call check_cpuid
  call check_long_mode
  call set_up_page_tables
  call enable_paging
  ; both LME and paging are enabled, we actually switch to long mode
  lgdt [gdt64.pointer]

  mov esi, stack_top
  mov edx, stack_bottom
  jmp gdt64.code:long_mode_start ; load gdt64.code into cs
  mov al, "3"
  jmp error

check_multiboot:
  cmp eax, 0x36d76289 ; grub pushed magic number into eax
  jne .no_multiboot
  ret

.no_multiboot:
  mov al, "0" ; we use al to save error code, set error code to 0
  jmp error

check_cpuid: ;check if cpuid instruction is supported
  pushfd ;since in x86, there is no way to save FLAGS directly to EAX, we need to first save it to stack
  pop eax ; copy FLAGS into EAX

  mov ecx, eax ; Copy to ecx
  xor eax, 1 << 21 ; flip bit 21 in eax
  
  push eax ; for the same reason as pushfd
  popfd ; copy content in eax to flags

  pushfd ; for the same reason as pushfd
  pop eax ; copy FLAGS into EAX

  cmp eax, ecx; if we have cpu_id instruction, the fliped bit should persist during above instructions, otherwise, the FLAGS register will be revert back to its old state
  je .no_cpuid ; if eax==ecx, means FLAGS register revert back to its old state, no cpu_id
  ret

.no_cpuid:
  mov al, "1" ; set error message
  jmp error

check_long_mode:
  mov eax, 0x80000000 ; set parameter as 0x80000000 for cpuid, return highest extened function number in eax
  cpuid ;
  cmp eax, 0x80000001 ; compare highest extened function number  with 0x80000001
  jb .no_long_mode ; if < 0x80000001 this means there is definitely no support for long mode

  mov eax, 0x80000001 ; set parameter as 0x80000001 for cpuid, returns extended feature flags in EDX and ECX.
  cpuid
  test edx, 1 << 29 ; bit 29 is set if the CPU supports long mode
  jz .no_long_mode
  ret

.no_long_mode:
  mov al, "2" ; set error message
  jmp error

; virtual address in x86-64 is devided as below
;| 63-48 | 47-39 | 38-30 | 29-21 | 20-12 |  11-0   |
;| Sign  | P4    |  P3   |  p2   |   P1  |  Offset |

set_up_page_tables:
  ; first P4 entry point to P3 table
  mov eax, p3_table
  or eax, 0b11 ; present and writable
  mov [p4_table], eax

  ; 511th P4 entry recurses to itself
  ; the reason is well explained in https://os.phil-opp.com/paging-implementation/
  mov eax, p4_table
  or eax, 0b11 ; present + writable
  mov [p4_table + 511 * 8], eax

  ; map first P3 entry to P2 table
  mov eax, p2_table
  or eax, 0b11 ; present + writable
  mov [p3_table], eax

  mov ecx, 0 ; counter
.map_p2_table:
  ; all 512 entry in p2_table are mapped to 2MB memory(large page)
  mov eax, 0x200000 ; 2MiB
  mul ecx           ; multiple eax by ecx and store in eax
  or eax, 0b10000011
  mov [p2_table + ecx * 8], eax ; map ecx entry

  inc ecx
  cmp ecx, 512
  jne .map_p2_table

  ret

enable_paging:
  mov eax, p4_table
  mov cr3, eax ; set cr3 point to p4_table

  mov eax, cr4
  or eax, 1 << 5 ; Physical Address Extension flag, The 5th bit of the CR4 register corresponds to the PAE flag

  mov cr4, eax ; PAE flag is set, this allows cpu to access more than 4 GB of physical memory

  ; set long mode bit in EFER model specific register
  ; EFER (Extended Feature Enable Register) is a Model Specific Register (MSR) that contains various flags related to extended CPU features. 
  mov ecx, 0xC0000080 ; mov addr of EFER register
  rdmsr ; read msr ; check this link for detail : https://www.felixcloutier.com/x86/rdmsr
  or eax, 1 << 8 ; set 8th bit which singaling cpu we want enable LME
  wrmsr ; write msr

  mov eax, cr0 ; detail of cr0: https://wiki.osdev.org/CPU_Registers_x86#CR0
  or eax, 1 << 31 ; 31st bit of the CR0 register is the PG (Paging) flag, we set this bit to enable paging
  mov cr0, eax
  ret

error:
  mov dword [0xb8000], 0x4f524f45 ; write to VGA
  mov dword [0xb8004], 0x4f3a4f52
  mov dword [0xb8008], 0x4f204f20
  mov byte  [0xb800a], al
  hlt

section .rodata
gdt64:
  dq 0
.code : equ $ - gdt64 ; labels addr is offset to code segment
  dq 0x20980000000000 ; suitable value for x86-64 of gdt
.pointer:
  dw $ - gdt64 - 1 ; length of gdt
  dq gdt64 ; ptr to gdt addr

section .bss ;  declaring variables that are uninitialized at the start of the program
align 4096
p4_table:
  resb 4096
p3_table:
  resb 4096
p2_table:
  resb 4096
p1_table:
  resb 4096
stack_bottom:
  resb 4096 * 6
stack_top:
