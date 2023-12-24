//!
//! Code for getting to userspace
//!
use multiboot2::BootInformation;
use object::{Object, ObjectSegment, File};
use crate::memory::paging::frameallocator::FrameAllocator;
use crate::memory::paging::entry::EntryFlags;
use crate::memory::{change_map_memory, map_memory, map_memory_expect};
use crate::gdt::GDT;

/// Module loaded by multiboot2 compliant bootloader
#[derive(Debug)]
pub struct Module {
    pub start_address : *const u8,
    pub size : usize
}

impl Module {
    /// Try to create module by checking if it is an elf file 
    pub fn init(boot_info : &BootInformation) -> Option<Module> {
        if let Some(module) = boot_info.module_tags().nth(0) {
            let module_start_address = module.start_address() as *const u8; 
            let module_size = (module.end_address() - module.start_address()) as usize;
            let module_slice = unsafe { core::slice::from_raw_parts(module_start_address, module_size) };
            if module_slice[0..4] == ELF_MAGIC {
                Some(Module{
                    start_address: module_start_address,
                    size : module_size
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Create an executable
    pub fn to_exe_object(self) -> InitExec<'static> {
        InitExec::new(self)
    }
}

/// A executable to init the OS
#[derive(Debug)]
pub struct InitExec<'a> {
    size : usize,
    exe : Option<File<'a>>
}

/// User code
pub struct UserCode {
    stack_end : usize,
    code_ptr : usize,
}

impl UserCode {
    /// Do the jump to userspace to the user code
    pub fn switch_to_userspace(self) -> ! {

        println!("Switching to userspace");
        println!("Code ptr: 0x{:x}", self.code_ptr);

        use x86_64::instructions::segmentation::{DS, Segment};
        use core::arch::asm;

        let (mut cs, mut ds) = (GDT.1.user_code_selector, GDT.1.user_data_selector);

        cs.set_rpl(x86_64::PrivilegeLevel::Ring3);
        ds.set_rpl(x86_64::PrivilegeLevel::Ring3);

        unsafe {
            DS::set_reg(ds);

            asm!(
                "cli",
                "push {:r}",   // stack segment
                "push {:r}",   // rsp
                "push 0x200",  // rflags with interrupt bit set
                "push {:r}",   // code segment
                "push {:r}",   // return to virtual addr
                "iretq",
                in(reg) ds.0,
                in(reg) self.stack_end,
                in(reg) cs.0,
                in(reg) self.code_ptr);
        }

        crate::hlt_loop();
    }
}

/// Elf magic number
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

impl<'a> InitExec<'a> {
    /// Create a new init executable from the module
    fn new(relocated : Module) -> InitExec<'a> {
        let module_slice = unsafe { core::slice::from_raw_parts(relocated.start_address, relocated.size) };
        if let Ok(result) = object::File::parse(module_slice) {
            InitExec {
                size : relocated.size,
                exe : Some(result)
            }   
        } else {
            InitExec {
                size : 0,
                exe : None
            }
        }
    }

    fn size_of_code(&self) -> usize {
        if let Some(obj) = &self.exe {
            
            let mut min = usize::MAX;
            let mut max = 0;

            // Map in executable to code
            for segment in obj.segments() {
               
                let addr = segment.address() as usize;
                let size = segment.size() as usize;
                
                if addr < min {
                    min = addr;
                }
                if addr > max {
                    max = addr;
                }
                if addr + size > max {
                    max = addr + size;
                }
            }

            if min == usize::MAX {
                0
            } else {
                max - min
            }

        } else {
            0
        }
    }

    /// Setup the stack and code for userspace    
    pub fn setup<A>(&self, allocator : &mut A) -> Result<UserCode, ()>
        where A: FrameAllocator 
    {

        if let Some(obj) = &self.exe {

            let mut code_ptr : *mut u8 = 0x40000 as *mut u8;
            let mut stack_ptr : *mut u8 = 0x80000 as *mut u8;
            let stack_size = 4096 * 6;
            
            serial_infoln!("Code size is {}", self.size_of_code());

            code_ptr = unsafe { map_memory(code_ptr as usize, self.size_of_code(), EntryFlags::USER_ACCESSIBLE | EntryFlags::WRITABLE, allocator) as *mut u8 };

            serial_debugln!("Mapped in code from {:x} to {:x}", code_ptr as usize, code_ptr as usize + self.size - 1);

            let before_stack_ptr = unsafe { map_memory(stack_ptr as usize - 4096, stack_size + 2 * 4096, EntryFlags::USER_ACCESSIBLE | EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE, allocator) };

            unsafe { change_map_memory(before_stack_ptr, 4093, EntryFlags::NO_EXECUTE); }
            unsafe { change_map_memory(before_stack_ptr + stack_size + 4096, 4093, EntryFlags::NO_EXECUTE); }

            stack_ptr = (before_stack_ptr + 4096) as *mut u8;
            serial_debugln!("Mapped in stack at {:x} to {:x}", stack_ptr as usize, stack_ptr as usize + stack_size);

            let entry_point = obj.entry();

            // Map in executable to code
            for segment in obj.segments() {
                let addr = segment.address() as usize;
                serial_debugln!("Offset is {}", addr);
                if let Ok(data) = segment.data() {
                    for (i, b) in data.iter().enumerate() {
                        unsafe { core::ptr::write(code_ptr.add(addr + i), *b) };
                    }
                }
            }

            // https://intezer.com/blog/malware-analysis/executable-and-linkable-format-101-part-3-relocations/
            if let Some(iter) = obj.dynamic_relocations() {
                for (addr, rel) in iter {

                    if rel.kind() == object::RelocationKind::Elf(8) {
                        serial_debugln!("{:?}", rel);
                        serial_debugln!("Found relative relocation");
                        serial_debugln!("Target is {:?}", rel.target());
                        serial_debugln!("Means Image base ({:x}) + Addend ({:x}) to {:x} into the code", code_ptr as usize, rel.addend(), addr);
                        let val : u64 = code_ptr as u64 + rel.addend() as u64;
                        let where_to_write = unsafe { code_ptr.add(addr as usize) as *mut u64 };
                        unsafe { core::ptr::write(where_to_write, val); }
                        // write 
                    } else {
                        serial_errorln!("Found relocation we cannot handle {:?}", rel.kind());
                        panic!("Unable to get to module");
                    }
                }
            }

            serial_debugln!("Creating heap");
            let heap_ptr : *mut u8 = 0x10000000 as *mut u8;
            let heap_size : usize = 4096 * 4;

            let heap_ptr_tmp = unsafe { map_memory_expect(heap_ptr as usize, heap_size, EntryFlags::USER_ACCESSIBLE | EntryFlags::WRITABLE, allocator) };
            if heap_ptr_tmp == Err(()) {
                panic!("Unable to create heap for userspace");
            }

            serial_debugln!("entry point is 0x{:x} which is at 0x{:x}", entry_point, code_ptr as usize + entry_point as usize);
            
            Ok(UserCode {
                code_ptr : code_ptr as usize + entry_point as usize,
                stack_end: stack_ptr as usize + stack_size,
            })
        } else {
            Err(())
        }
    }
}

/// Go to init process in module with boot info and frame allocator to create
/// pages
pub fn initproc<A>(boot_info : &BootInformation, frame_allocator : &mut A) -> !
    where A: FrameAllocator
{
    let module = Module::init(boot_info).expect("Expect module");
    println!("{:?}", module);
    let exe = module.to_exe_object();
    let user_code = exe.setup(frame_allocator).expect("Successful setup"); 
    user_code.switch_to_userspace()
}

