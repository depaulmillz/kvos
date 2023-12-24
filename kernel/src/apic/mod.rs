use core::arch::asm;
use crate::acpi::IOAPICInfo;
use x86_64::registers::model_specific::Msr;
use core::ptr::{read_volatile, write_volatile};
use crate::memory::paging::ActivePageTable;
use crate::memory::paging::translation::{Frame, Page};
use crate::memory::paging::frameallocator::FrameAllocator;
use crate::memory::paging::entry::EntryFlags;
use core::sync::atomic::{AtomicUsize, Ordering};

pub fn has_apic() -> bool {

    let mut x : u64;
    unsafe {
        asm!(
            "mov eax, 0x1",
            "cpuid",
            "mov {x}, rdx",
            x = out(reg) x,
            );
    }
    (x >> 9) & 0x1 == 1
}

pub struct IOAPIC {
    ptr : AtomicUsize
}

impl IOAPIC {

    pub const fn zeroed() -> IOAPIC {
        let ptr = AtomicUsize::new(0);
        IOAPIC { 
            ptr
        }
    }

    pub fn write_reg(&self, offset : u8, val : u32) {
        let apic_base = self.ptr.load(Ordering::Relaxed) as *mut u32;
        serial_debugln!("Apic base is {:p} and IOREGWIN is {:p}", apic_base, apic_base.wrapping_add(4));
        unsafe {
            write_volatile(apic_base, offset as u32); // IOREGSEL
            write_volatile(apic_base.wrapping_add(4), val); // IOREGWIN
        }
    }

    pub fn read_reg(&self, offset : u8) -> u32 {
        let apic_base = self.ptr.load(Ordering::Relaxed) as *mut u32;
        serial_debugln!("Apic base is {:p} and IOREGWIN is {:p}", apic_base, apic_base.wrapping_add(4));
        unsafe {
            write_volatile(apic_base, offset as u32);
            read_volatile(apic_base.wrapping_add(4))
        }
    }

    pub fn init<A>(&self, ioapic: IOAPICInfo, lapic_id: u32, alloc : &mut A) where A: FrameAllocator {
        // Map APIC into memory
        serial_debugln!("Remaping IOAPIC into memory {:x}", ioapic.addr as usize);
        let mut pt = unsafe { ActivePageTable::new() };
        pt.map_to(Page::containing_address(ioapic.addr as usize), Frame::containing_address(ioapic.addr as usize), EntryFlags::WRITABLE | EntryFlags::NO_CACHE, alloc);
        
        self.ptr.store(ioapic.addr as usize, Ordering::SeqCst);

        self.write_reg(0x12, 33); // vector
        serial_debugln!("R12 is {:x}", self.read_reg(0x12));
        self.write_reg(0x13, lapic_id << 24); // local apic id
        serial_debugln!("R13 is {:x}", self.read_reg(0x13));
        serial_println!("Setting reg 13 to {:x} where local apic id is {:x}", lapic_id << 24, lapic_id);
        serial_debugln!("R12 is {:x} and R13 is {:x}", self.read_reg(0x12), self.read_reg(0x13));
    }
}

pub struct LAPIC {
   ptr : AtomicUsize
}

impl LAPIC {

    pub const fn zeroed() -> LAPIC {

        let ptr = AtomicUsize::new(0);

        LAPIC {
            ptr
        }
    }

    pub fn get_ptr(&self) -> usize {
        self.ptr.load(Ordering::Relaxed)
    }

    pub fn get_apic_id(&self) -> usize {
        let apic : *const usize = self.get_ptr() as *const usize;
        unsafe { read_volatile(apic.add(0x20)) }
    }

    pub fn init<A>(&self, alloc : &mut A) where A: FrameAllocator {
        // rdmsr APIC_BASE
        //
        // boot strap processor flag bit 8 must be set to 1 (10.4.4 x86 apic)
        // APIC global enable flag in bit 11 must be set
        // APIC base field, bits 12 to 35 specififes base address of APIC registers

        // LVT local vector table must be created
        // specifies how local interrupts are delivered to core
        //
        // init timer register, lint0 register, lint1 register
        // others may be there
        //
        // lvt has registers can have an interrupt vector number
        // and a delivery mode

        // inter processor interrupts are future work
        // Constants used in timer setup; this is here for reference.
        // https://wiki.osdev.org/APIC

        //     -----------------------------------------------------------------------------------------
        //     |    APIC_APICID      |    0x020    |  Not used here                                    |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_APICVER     |    0x030    |  Not used here                                    |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_TASKPRIOR   |    0x080    |  task priority                                    |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_EOI         |    0x0B0    |  End of Interrupt (EOI)                           |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LDR         |    0x0D0    |  Logical Destination Register (LDR)               |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_DFR         |    0x0E0    |  Destination Format Register (DFR)                |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_SPURIOUS    |    0x0F0    |  Spurious Interrupt Vector register (SIVR)        |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_ESR         |    0x280    |  Error Status Register (ESR)                      |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_ICRL        |    0x300    |  Interrupt Command Register (ICRL)                |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_ICRH        |    0x310    |  Interrupt Command Register (ICRH)                |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LVT_TMR     |    0x320    |  LVT Timer Register                               |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LVT_PERF    |    0x340    |  LVT Performance Monitoring Counters Register     |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LVT_LINT0   |    0x350    |  LINT0 Register                                   |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LVT_LINT1   |    0x360    |  LINT1 Register                                   |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LVT_ERR     |    0x370    |  Error Register                                   |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_TMRINITCNT  |    0x380    |  Initial Count Register (timer)                   |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_TMRCURRCNT  |    0x390    |  Current Count Register (timer)                   |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_TMRDIV      |    0x3E0    |  Divide Configuration Register (timer)            |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_LAST        |    0x38F    |  Not used here                                    |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_DISABLE     |    0x10000  |  Used in init                                     |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_SW_ENABLE   |    0x100    |  Used in enabling PIC                             |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_CPUFOCUS    |    0x200    |  Not used here                                    |
        //     -----------------------------------------------------------------------------------------
        //     |    APIC_NMI         |    4<<8     |  Used in init                                     |
        //     -----------------------------------------------------------------------------------------
        //     |    TMR_PERIODIC     |    0x20000  |  Used to re-enable timer in periodic mode         |
        //     -----------------------------------------------------------------------------------------
        //     |    TMR_BASEDIV      |    1<<20    |  Not used here                                    |
        //     -----------------------------------------------------------------------------------------

        let base_msr : u32 = 0x1B;

        let mut lapic_msr = Msr::new(base_msr);

        let mut tmp = unsafe { lapic_msr.read() };

        assert!(tmp & 0x800 != 0); // APIC global enable flag
        assert!(tmp & 0x100 != 0); // boot strap processor flag

        println!("Read msr {:x}", tmp);

        let ptr : *mut u8 = (tmp & 0xfffff000) as *mut u8; 

        // Map APIC into memory
        let mut pt = unsafe { ActivePageTable::new() };
        pt.map_to(Page::containing_address(ptr as usize), Frame::containing_address(ptr as usize), EntryFlags::WRITABLE | EntryFlags::NO_CACHE, alloc);
        println!("{:x} to {:?}", ptr as usize, pt.translate(ptr as usize));

        // enable xAPIC and x2APIC
        tmp = tmp | 0x800;

        unsafe { lapic_msr.write(tmp); }

        // Spurious Interrupt
        unsafe {
            write_volatile(ptr.add(0xF0) as *mut u32, 0x100 | 39);      // enable spurious interrupt
        };

        // Timer
        unsafe {
            write_volatile(ptr.add(0x3e0) as *mut u32, 0);              // init timer division
            write_volatile(ptr.add(0x380) as *mut u32, 0xffff);         // init timer count
            write_volatile(ptr.add(0x320) as *mut u32, 0x20000 | 32);   // enable timer in periodic mode, and enable timer interrupt
        }

        self.ptr.store(ptr as usize, Ordering::SeqCst);

        serial_debugln!("APIC inited");
    }

    pub fn init_cpu(&self, apic_id : u8, addr : u16) {
        // addr is a 16 bit pointer since the processor starts up in real mode
        use x86_64::instructions::port::Port;
        let ptr = self.get_ptr() as *mut u8;
        unsafe {
            // CMOS shutdown code
            Port::<u8>::new(0x70).write(0xF);
            Port::<u8>::new(0x71).write(0x0A);

            let warm_reset_vector_ptr = (0x40<<4 | 0x67) as *mut u16;
            write_volatile(warm_reset_vector_ptr, 0);
            write_volatile(warm_reset_vector_ptr.wrapping_add(1), addr >> 4);

            // Now initialize the processor through the local apic registers
            write_volatile(ptr.add(0x310) as *mut u32, (apic_id as u32) << 24);
            write_volatile(ptr.add(0x300) as *mut u32, 0x00000500 | 0x00008000 | 0x00004000); // write init level and assert flags
            write_volatile(ptr.add(0x300) as *mut u32, 0x00000500 | 0x00008000); // write init and level

            // send two inits
            for _ in 0..2 {
                write_volatile(ptr.add(0x310) as *mut u32, (apic_id as u32) << 24);
                write_volatile(ptr.add(0x300) as *mut u32, 0x00000600 | (addr as u32) >> 12); // write startup and addr
            }
        }
    }

    pub fn eoi(&self) {
        unsafe {
            let ptr = self.ptr.load(Ordering::Relaxed) as *mut u8;
            write_volatile(ptr.add(0xB0) as *mut u32, 0);
        }
    }
}

/*

this is not used.  It has been left in only for understanding what is going on
above, specifically the assembly.  The assembly is partially wrong (there's no
memory mapping being done in it, which is necessary), but it does help explain
what is going on.

pub fn init_local_apic() {
    let (mut apic, mut b, mut msr, mut hi) : (u64, u64, u64, u64);
    unsafe {
       asm!(
           "rdmsr",
           "mov    {apic},     rcx", // APIC Base
           "mov    {b},        rbx", // ???
           "mov    {msr},      rcx", // Probably unnecessary.
           "mov    {hi},       rdx", // Probably unnecessary.

           // disable pic
           "mov al, 0xff",
           "out 0xa1, al",
           "out 0x21, al",

           // set boot strap processor flag
           "mov    dword ptr [{apic} + 0x008], 1",
           // set APIC global enable flag
           "mov    dword ptr [{apic} + 0x00B], 1",

           // https://wiki.osdev.org/APIC_Timer

           
           // Since we have a working IDT that includes a timer interrupt,
           // I'm fairly certain this section is unnecessary.  However, I
           // have left it in, just in case.
           
           // set up ISRs
           // "mov    al,         32" ,
           // "mov    ebx,        {handler}",     // this should be the timer interrupt handler from
                                               // our interrupts.rs
           /*
           "call   writegate", // This is an assumed thing to "write a gate for a specific interrupt"
           "mov    al,         39" ,
           "mov    ebx,        isr_spurious",  // this is used to "set up a specific interrupt gate in IDT"
                                               // Once again, not sure what to do with this.
           "call   writegate",

           */

           // ------------------------------------------------------------------------------------------

           // Initialize Local APIC (LAPIC) to a well-known state
           
           "mov    dword ptr [{apic} + $0x0E0], 0x0FFFFFFFF",  //APIC_DFR
           "mov    eax, [{apic} + 0x0D0]",                     // APIC_LDR
           "and    eax, 0x00FFFFFF",
           "or     al, 1",
           "mov    [{apic} + 0x0D0], eax",                     // APIC_LDR
           "mov    dword ptr [{apic} + 0x320], 0x10000",       // APIC_LVT_TMR, APIC_DISABLE
           "mov    dword ptr [{apic} + 0x340], 4 << 8",        // APIC_LVT_PERF, APIC_NMI
           "mov    dword ptr [{apic} + 0x350], 0x10000",       // APIC_LVT_LINT0, APIC_DISABLE
           "mov    dword ptr [{apic} + 0x360], 0x10000",       // APIC_LVT_LINT1, APIC_DISABLE
           "mov    dword ptr [{apic} + 0x080], 0",             // APIC_TASKPRIOR
           // enable apic
           // global
           "mov    ecx, 0x1B",
           "rdmsr",
           "bts    eax, 11",
           "wrmsr",
           // Software enable, map spurious interrupt to ISR
           "mov    dword ptr [{apic} + 0x0F0], 39 + 0x100",    // APIC_SPURIOUS, APIC_SW_ENABLE
           // Map timer to interrupt, therefore enabling it in one-shot mode
           "mov    dword ptr [{apic} + 0x320], 32",            // APIC_LVT_TMR
           // Set divide value to 16
           "mov    dword ptr [{apic} + 0x3E0], 0x10",          // APIC_TMRDIV; OSDEV APIC Timer has 0x03 (03h)?
           // ebx = 0xFFFFFFFF
           "xor    ebx, ebx",
           "dec    ebx",
           // ^^ I'm unsure if these instructions are necessary?  Since they theoretically just zero out ebx

           // ------------------------------------------------------------------------------------------

           // initialize PIT Ch 2 in one-shot mode
           // Wait 1/100 sec, multiply counted ticks
           "mov    dx, 0x61",
           "in     al, dx",
           "and    al, 0X0FD",
           "or     al, 1",
           "out    dx, al",
           "mov    al, 0xB2", // 0b10110010
           "out    0x43, al",
           // 1193180/100 Hz = 11931 = 2e9bh
           // ^^ what?  Where is this coming from?  It's in the OSDEV page, but I'm unsure where they're
           //    getting this number from
           "mov   al, 0x9B",  // LSB
           "out   0x42, al",
           "in    al, 0x60",  // short delay
           "mov   al, 0x2e",  // MSB
           "out   0x42, al",
           // reset PIT one-shot counter (start counting)
           "in    al, dx",
           "and   al, 0x0FE",
           "out   dx, al", // gate low
           "or    al, 1",
           "out   dx, al",  // gate high
           // reset APIC timer (set counter to -1)
           "mov   dword ptr [{apic} + 0x380], ebx",            // APIC_TMRINITCNT
           // now wait until PIT counter reaches zero
           "in    al, dx", // This line has "@@:" before it
           "and   al, 0x20",
           // "jz   @b",
           // stop APIC timer
           "mov   dword ptr [{apic} + 0x320], 0x10000",        // APIC_LVT_TMR, APIC_DISABLE
           // now do the math...
           "xor   eax, eax",
           "xor   ebx, ebx",
           "dec   eax",
           // get current counter value
           "mov   ebx, dword ptr [{apic} + 0x390]",            // APIC_TMRCURRCNT
           // it is counted down from -1, make it positive
           "sub   eax, ebx",
           "inc   eax",
           // we used divide value different than 1, so now we have to multiply the result by 16
           "shl   eax, 4",  // *16
           "xor   edx, edx",
           // moreover, PIT did not wait a whole sec, only a fraction, so multiply by that too
           "mov   ebx, 100",  // *PITHz
           "mul   ebx",

           // ------------------------------------------------------------------------------------------

           // edx:eax now holds the CPU bus frequency
           
           // now calculate timer counter value of your choice
           // this means that tasks will be preempted 1000 times in a second. 100 is popular too.
           "mov   ebx, 1000",
           "xor   edx, edx",
           "div   ebx",
           // again, we did not use divide value of 1
           "shr   eax, 4",  // /16
           // sanity check, min 16
           "cmp   eax, 0x010",
           // "jae   @f",
           "mov   eax, 0x010",
           // now eax holds appropriate number of ticks, use it as APIC timer counter initializer
           "mov   dword ptr [{apic} + 0x380], eax",            // APIC_TMRINITCNT; this line has @@: before it
           // finally re-enable timer in periodic mode
           "mov   dword ptr [{apic} + 0x320], 0x20000",        // APIC_LVT_TMR, TMR_PERIODIC
           "mov   dword ptr [{apic} + 0x3E0], 0x10",           // APIC_TMRDIV; Once again, OSDEV uses 0x03?

           // ------------------------------------------------------------------------------------------

           apic     = out(reg) apic     ,
           b        = out(reg) b        ,
           msr      = out(reg) msr      ,
           hi       = out(reg) hi       ,
       );
    }
    println!("Local Apic enabled!");

    // "The local APIC registers are memory mapped to an address 
    //  that can be found in the MP/MADT tables. Make sure you 
    //  map these to virtual memory if you are using paging."
    
    // "Each register is 32 bits long, and expects to be written
    //  and read as a 32 bit integer. Although each register is
    //  4 bytes, they are all aligned on a 16 byte boundary."
}

*/
