// modified from MOROS Rust OS: https://github.com/vinc/moros
// The MIT License (MIT)
//
//  Copyright (c) 2019-2022 Vincent Ollivier
//  
//  Permission is hereby granted, free of charge, to any person obtaining a copy
//  of this software and associated documentation files (the "Software"), to deal
//  in the Software without restriction, including without limitation the rights
//  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//  copies of the Software, and to permit persons to whom the Software is
//  furnished to do so, subject to the following conditions:
//  
//  The above copyright notice and this permission notice shall be included in
//  all copies or substantial portions of the Software.
//  
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
//  THE SOFTWARE.
use alloc::string::String;
use alloc::vec::Vec;
use bit_field::BitField;
use core::{convert::TryInto, hint::spin_loop};
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use crate::serial_println;
use crate::drivers::timing;

// See "Information Technology - AT Attachment with Packet Interface Extension (ATA/ATAPI-4)" (1998)

//this block size is important and is set in the QEMU settings
pub const BLOCK_SIZE: usize = 512;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
}

enum IdentifyResponse {
    Ata([u32; 128]),
    Atapi,
    Sata,
    None,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum Status {
    ERR  = 0, // Error
    IDX  = 1, // (obsolete)
    CORR = 2, // (obsolete)
    DRQ  = 3, // Data Request
    DSC  = 4, // (command dependant)
    DF   = 5, // (command dependant)
    DRDY = 6, // Device Ready
    BSY  = 7, // Busy
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
//all the ports to read/write from
pub struct Bus {
    id: u8,
    irq: u8,
    data_register: Port<u32>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,
    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockess_register: PortReadOnly<u8>,
}

impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id, irq,
            data_register: Port::new(io_base + 0),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),
            alternate_status_register: PortReadOnly::new(ctrl_base + 0),
            control_register: PortWriteOnly::new(ctrl_base + 0),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    fn check_floating_bus(&mut self) -> Result<(), ()> {
        match self.status() {
            0xFF | 0x7F => Err(()),
            _ => Ok(()),
        }
    }

    fn wait(&mut self) {
        timing::nanosleep(400);
    }

    fn clear_interrupt(&mut self) -> u8 {
        unsafe { self.status_register.read() }
    }

    fn status(&mut self) -> u8 {
        unsafe { self.alternate_status_register.read() }
    }

    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    fn read_data(&mut self) -> u32 {
        unsafe { self.data_register.read() }
    }

    fn write_data(&mut self, data: u32) {
        unsafe { self.data_register.write(data) }
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }
    //polls the port to make sure the command is done
    fn poll(&mut self, bit: Status, val: bool) -> Result<(), ()> {
        let start = timing::get_ticks();
        while self.status().get_bit(bit as usize) != val {
            if timing::get_ticks() - start > 10 {
                serial_println!("ATA hanged while polling {:?} bit in status register", bit);
                self.debug();
                return Err(());
            }
            spin_loop();
        }
        spin_loop();
        Ok(())
    }
    //gets the drive
    fn select_drive(&mut self, drive: u8) -> Result<(), ()> {
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        unsafe {
            // Bit 4 => DEV
            // Bit 5 => 1
            // Bit 7 => 1
            self.drive_register.write(0xA0 | (drive << 4))
        }
        self.wait();
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;
        Ok(())
    }
    //get the ATA ready to receive the cmd
    fn write_command_params(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);
        unsafe {
            self.sector_count_register.write(1);
            self.lba0_register.write(bytes[0]);
            self.lba1_register.write(bytes[1]);
            self.lba2_register.write(bytes[2]);
            self.drive_register.write(bytes[3]);
        }
        Ok(())
    }
    //writes the command to the command register
    fn write_command(&mut self, cmd: Command) -> Result<(), ()> {
        unsafe { self.command_register.write(cmd as u8) }
        self.wait();
        self.status(); // Ignore results of first read
        self.clear_interrupt();
        if self.status() == 0 { // Drive does not exist
            return Err(());
        }
        if self.is_error() {
            self.debug();
            serial_println!("Write Command Error");
            return Err(());
        }
        //ensure the command was receiv
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, true)?;
        Ok(())
    }
    //set up the PIO to have a cmd written
    fn setup_pio(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        self.select_drive(drive)?;
        self.write_command_params(drive, block)?;
        Ok(())
    }

    fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) -> Result<(), ()> {
        if buf.len() != BLOCK_SIZE {
            return Err(());
        }
        self.setup_pio(drive, block)?;
        //writes the read command
        self.write_command(Command::Read)?;
        //reads data from the port into the buffer
        for chunk in buf.chunks_mut(4) {
            let data = self.read_data().to_le_bytes();
            chunk.clone_from_slice(&data);
        }
        if self.is_error() {
            serial_println!("ATA read: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }
    //write the ATA device
    fn write(&mut self, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
        //ensure the buf has the length of the block size
        if buf.len() != BLOCK_SIZE {
            return Err(());
        }
        self.setup_pio(drive, block)?;
        //write the write command
        self.write_command(Command::Write)?;
        //writes the buf to the port to be written to the disk
        for chunk in buf.chunks(4) {
            let data = u32::from_le_bytes(chunk.try_into().unwrap());
            self.write_data(data);
        }
        if self.is_error() {
            serial_println!("ATA write: data error");
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }
    //identifys the drive and gets it ready to function
    fn identify_drive(&mut self, drive: u8) -> Result<IdentifyResponse, ()> {
        if self.check_floating_bus().is_err() {
            return Ok(IdentifyResponse::None);
        }
        self.select_drive(drive)?;
        self.write_command_params(drive, 0)?;
        if self.write_command(Command::Identify).is_err() {
            if self.status() == 0 {
                return Ok(IdentifyResponse::None);
            } else {
                return Err(());
            }
        }
        match (self.lba1(), self.lba2()) {
            (0x00, 0x00) => Ok(IdentifyResponse::Ata([(); 128].map(|_| { self.read_data() }))),
            (0x14, 0xEB) => Ok(IdentifyResponse::Atapi),
            (0x3C, 0xC3) => Ok(IdentifyResponse::Sata),
            (_, _) => Err(()),
        }
    }
    //reset the ATA device
    #[allow(dead_code)]
    fn reset(&mut self) {
        unsafe {
            self.control_register.write(4); // Set SRST bit
            self.wait();                   // Wait at least 5 ns
            self.control_register.write(0); // Then clear it
            self.wait();                // Wait at least 2 ms
        }
    }
    //print the ATA status/error registers
    #[allow(dead_code)]
    fn debug(&mut self) {
        unsafe{
            serial_println!("ATA status register: 0b{:08b} <BSY|DRDY|#|#|DRQ|#|#|ERR>", self.alternate_status_register.read());
            serial_println!("ATA error register:  0b{:08b} <#|#|#|#|#|ABRT|#|#>", self.error_register.read());
        }
    }
}
//lock for the bus
lazy_static! {
    pub static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}
//init a new bus
pub fn init() {
    let mut buses = BUSES.lock();
    buses.push(Bus::new(0, 0x1F0, 0x3F6, 14));
    buses.push(Bus::new(1, 0x170, 0x376, 15));
}

#[derive(Clone)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    blocks: u32,
    model: String,
    serial: String,
}

impl Drive {
    //opens a new drive
    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        //lock the buses
        let mut buses = BUSES.lock();
        if let Ok(IdentifyResponse::Ata(res)) = buses[bus as usize].identify_drive(dsk) {
            let buf = res.map(u32::to_be_bytes).concat();
            let serial = String::from_utf8_lossy(&buf[20..40]).trim().into();
            let model = String::from_utf8_lossy(&buf[54..94]).trim().into();
            let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);
            Some(Self { bus, dsk, model, serial, blocks })
        } else {
            None
        }
    }
    //gets the block size
    pub const fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }
    //gets the number of blocks
    pub fn block_count(&self) -> u32 {
        self.blocks
    }
    //makes size of drive human readable
    fn humanized_size(&self) -> (usize, String) {
        let size = self.block_size() as usize;
        let count = self.block_count() as usize;
        let bytes = size * count;
        if bytes >> 20 < 1000 {
            (bytes >> 20, String::from("MB"))
        } else {
            (bytes >> 30, String::from("GB"))
        }
    }
}

//string result of the drives
impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

//gets a list of all the drives
pub fn list() -> Vec<Drive> {
    let mut res = Vec::new();
    for bus in 0..2 {
        for dsk in 0..2 {
            if let Some(drive) = Drive::open(bus, dsk) {
                res.push(drive)
            }
        }
    }
    res
}

//read from ATA
pub fn read(bus: u8, drive: u8, block: u32, buf: &mut [u8]) -> Result<(), ()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].read(drive, block, buf)
}

//write to ATA
pub fn write(bus: u8, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buf)
}
