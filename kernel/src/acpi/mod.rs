use core::result::Result;
use core::result::Result::{Ok, Err};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ACPISDTHeader {
    pub signature : [u8; 4],
    pub length : u32,
    pub revision : u8,
    pub checksum : u8,
    pub oem_id : [u8; 6],
    pub oem_table_id : [u8; 8],
    pub oem_revision : u32,
    pub creator_id : u32,
    pub creator_revision : u32
}

impl ACPISDTHeader {
    pub fn load(ptr: *const u8) -> ACPISDTHeader {
        let mut header = ACPISDTHeader{
            signature : [0, 0, 0, 0],
            length : 0,
            revision : 0,
            checksum : 0,
            oem_id : [0, 0, 0, 0, 0, 0],
            oem_table_id : [0, 0, 0, 0, 0, 0, 0, 0],
            oem_revision : 0,
            creator_id : 0,
            creator_revision : 0,
        };
        let ptr_to_header : *mut ACPISDTHeader = &mut header;
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, ptr_to_header as *mut u8, core::mem::size_of::<ACPISDTHeader>());
        }
        header
    }

    pub fn signature_str(&self) -> &str {
        let sig : &[u8] = &self.signature;
        core::str::from_utf8(sig).expect("Unable to convert to utf-8")
    }

}

#[derive(Debug)]
pub struct RSDT {
    ptr : *const u8,
    rsdt_header : ACPISDTHeader,
}

impl RSDT {
    pub fn new(ptr : *const u8) -> RSDT {
       
        let rsdt_header = ACPISDTHeader::load(ptr);
        
        RSDT { ptr, rsdt_header }
    }

    pub fn header(&self) -> ACPISDTHeader {
        self.rsdt_header
    }

    pub fn num_tables(&self) -> usize {
        (self.rsdt_header.length as usize - core::mem::size_of::<ACPISDTHeader>()) / 4
    }

    pub fn table(&self, i : usize) -> *const u8 {
        let pointers = self.ptr.wrapping_add(core::mem::size_of::<ACPISDTHeader>()) as *const u32; // location
                                                                                                   // of
                                                                                                   // pointers
        let mut table_ptr : u32 = 0;

        unsafe {
            let ptr_to_table_ptr : *mut u32 = &mut table_ptr;
            core::ptr::copy_nonoverlapping(pointers.wrapping_add(i) as *const u8, ptr_to_table_ptr as *mut u8, 4);
        }
        table_ptr as *const u8
    }

    pub fn checksum_valid(&self) -> bool {
        let mut checksum : u8 = 0;
        for i in 0..self.header().length as usize {
            checksum = checksum.wrapping_add(unsafe { *self.ptr.wrapping_add(i) });
        }
        checksum == 0
    }
}

#[derive(Debug)]
pub struct MADT {
    ptr : *const u8,
    madt_header : ACPISDTHeader,
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct IOAPICInfo {
    pub id : u8,
    pub reserved : u8,
    pub addr : u32,
    pub gsib : u32 // Global system interrupt base
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct LAPICInfo {
    pub proc_id : u8,
    pub apic_id : u8,
    pub flags : u32,
}

impl MADT {
    pub fn new(ptr : *const u8) -> MADT {
       
        let madt_header = ACPISDTHeader::load(ptr);
        
        MADT { ptr, madt_header }
    }

    pub fn header(&self) -> ACPISDTHeader {
        self.madt_header
    }
   
    pub fn lapic(&self) -> u32 {
        let mut lapic_ptr : u32 = 0;

        unsafe {
            let ptr_to_lapic_ptr : *mut u32 = &mut lapic_ptr;
            core::ptr::copy_nonoverlapping(self.ptr.wrapping_add(0x24) as *const u8, ptr_to_lapic_ptr as *mut u8, 4);
        }
        lapic_ptr
    }

    pub fn flags(&self) -> u32 {
        let mut flags : u32 = 0;

        unsafe {
            let ptr_to_flags : *mut u32 = &mut flags;
            core::ptr::copy_nonoverlapping(self.ptr.wrapping_add(0x28) as *const u8, ptr_to_flags as *mut u8, 4);
        }
        flags
    }

    pub fn num_entries(&self) -> usize {
        let mut ptr = self.ptr.wrapping_add(0x2c);
        let mut count = 0 as usize;
        while ptr < self.ptr.wrapping_add(self.header().length as usize) {
            let entry_type = unsafe { *ptr };
            let len = unsafe { *ptr.wrapping_add(1) };
            if entry_type == 2 { // FIXME io/apic interrupt source override needs to be handled if
                                 // the irq source is 1
                                 // we know this function will be run so we are just asserting that it
                                 // doesnt occur and we dont have to handle it
                                 // https://blog.wesleyac.com/posts/ioapic-interrupts
                let irq_source = unsafe { *ptr.wrapping_add(3) };
                assert!(irq_source != 1);
            }
            ptr = ptr.wrapping_add(len as usize);            
            count += 1;
        }
        count
    }

    pub fn ith_entry_type(&self, i : usize) -> u8 {
        let mut ptr = self.ptr.wrapping_add(0x2c);
        let mut count = 0;
        while ptr < self.ptr.wrapping_add(self.header().length as usize) {
            let entry_type = unsafe { *ptr };
            let len = unsafe { *ptr.wrapping_add(1) };
            ptr = ptr.wrapping_add(len as usize);
            if count == i {
                return entry_type;
            }
            count += 1;
        }
        panic!("Did not work");
    }

    pub fn get_ioapic_at_index(&self, i : usize) -> Result<IOAPICInfo, ()> {
        let mut ptr = self.ptr.wrapping_add(0x2c);
        let mut count = 0;
        while ptr < self.ptr.wrapping_add(self.header().length as usize) {
            let entry_type = unsafe { *ptr };
            let len = unsafe { *ptr.wrapping_add(1) };
            if count == i && entry_type == 1 {
                let mut info = IOAPICInfo {id : 0, reserved : 0, addr : 0, gsib : 0};
                 unsafe {
                    let ptr_to_info : *mut IOAPICInfo = &mut info;
                    core::ptr::copy_nonoverlapping(ptr.wrapping_add(2), ptr_to_info as *mut u8, core::mem::size_of::<IOAPICInfo>());
                }
                return Ok(info);
            }
            ptr = ptr.wrapping_add(len as usize);
            count += 1;
        }
        return Err(());
    }

    pub fn get_lapic_at_index(&self, i : usize) -> Result<LAPICInfo, ()> {
        let mut ptr = self.ptr.wrapping_add(0x2c);
        let mut count = 0;
        while ptr < self.ptr.wrapping_add(self.header().length as usize) {
            let entry_type = unsafe { *ptr };
            let len = unsafe { *ptr.wrapping_add(1) };
            if count == i && entry_type == 0 {
                let mut info = LAPICInfo {proc_id : 0, apic_id : 0, flags : 0};
                 unsafe {
                    let ptr_to_info : *mut LAPICInfo = &mut info;
                    core::ptr::copy_nonoverlapping(ptr.wrapping_add(2), ptr_to_info as *mut u8, core::mem::size_of::<LAPICInfo>());
                }
                return Ok(info);
            }
            ptr = ptr.wrapping_add(len as usize);
            count += 1;
        }
        return Err(());
    }

    pub fn checksum_valid(&self) -> bool {
        let mut checksum : u8 = 0;
        for i in 0..self.header().length as usize {
            checksum = checksum.wrapping_add(unsafe { *self.ptr.wrapping_add(i) });
        }
        checksum == 0
    }
}

pub struct CPUData {
    cpus : [Option<LAPICInfo>; 64], // support up to 64 cores
    size_ : usize,
}

impl CPUData {

    fn new() -> CPUData {
        CPUData {
            cpus : [None; 64],
            size_ : 0
        }
    }

    fn add(&mut self, info : LAPICInfo) {
        assert!(self.size_ < 64);
        self.cpus[self.size_] = Some(info);
        self.size_ += 1;
    }

    pub fn get(&self, idx : usize) -> Result<LAPICInfo, ()>  {
        if idx < self.size() {
            if let Some(cpu) = self.cpus[idx] {
                Ok(cpu)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn size(&self) -> usize {
        self.size_
    }
}

pub fn init(rsdt_addr : *const u8) -> Result<(IOAPICInfo, CPUData), ()> {
    let rsdt = RSDT::new(rsdt_addr);

    assert!(rsdt.checksum_valid());

    serial_infoln!("{:?}", rsdt);

    let mut cpus = CPUData::new();

    for i in 0..rsdt.num_tables() {
        let ptr = rsdt.table(i);
        let header = ACPISDTHeader::load(ptr);

        serial_infoln!("Table {} is {} : {:?}", i, header.signature_str(), header);
        
        if header.signature_str() == "APIC" {
            let madt = MADT::new(ptr);
            assert!(madt.checksum_valid());
            serial_infoln!("LAPIC addr {:x}", madt.lapic());
            serial_infoln!("Num entries {}", madt.num_entries());
            for entry in 0..madt.num_entries() {
                if madt.ith_entry_type(entry) == 0 {
                    let lapic = madt.get_lapic_at_index(entry).expect("Should work");
                    cpus.add(lapic);
                    serial_infoln!("LAPIC {:?}", lapic);
                }
            }
            for entry in 0..madt.num_entries() {
                if madt.ith_entry_type(entry) == 1 {
                    let ioapic = madt.get_ioapic_at_index(entry).expect("Should work");
                    serial_infoln!("{:p}", ioapic.addr as *const u8);
                    return Ok((ioapic, cpus));
                }
            }
        }
    }
    Err(())
}

