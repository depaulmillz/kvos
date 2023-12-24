extern crate alloc;
use alloc::vec::Vec;
use crate::disk::ata;
use crate::disk::ata::write;
use crate::disk::ata::read;
use crate::disk::ata::Drive;

//disk struct to store state of current disk
pub struct Disk {
    bus: u8,
    drive: u8,
    current_block: usize,
    current_address_in_block: usize,
}

impl Disk {
    //makes a new disk
    pub fn new(bus: u8, drive:u8) -> Self {
        ata::init();
        Drive::open(bus, drive);
        Self {
            bus,
            drive,
            current_block: 0,
            current_address_in_block: 0,
        }
    }
    //overwrites what ever is on disk currently
    //use to save whole map / compact log
    pub fn over_write_disk(&mut self, buf_to_write: Vec<u8>) -> Result<(), ()> {
        //get the length of the vec to write
        let length = buf_to_write.len();
        //for each 512 block of the vec write it to disk
        for i in 0..(length / 512) {
            let start_index = i * 512;
            let end_index = (i + 1) * 512;
            write(self.bus, self.drive, i as u32, &buf_to_write[start_index..end_index])?;
        }
        //save the current block and block adress so we can add logs
        self.current_block = length / 512;
        self.current_address_in_block = 0;
        //if the log does not fit on 512 byte boundary need to make it so it does
        if length % 512 != 0 {
            let last_block_size = length % 512;
            let last_buf: Vec<u8> = buf_to_write[(length - last_block_size)..].to_vec();
            //add 0 to the end of the vec
            let mut padded_last_buf: Vec<u8> = [0; 512].to_vec();
            padded_last_buf[..last_block_size].copy_from_slice(&last_buf);
    
            write(self.bus, self.drive, self.current_block as u32, &padded_last_buf)?;
            self.current_address_in_block = self.current_address_in_block + length %512;
        }
        //write a whole block of 0 to signify end of data
        let empty_buf = [0; 512].to_vec();
        write(self.bus, self.drive, (self.current_block +1) as u32, &empty_buf)?;
        Ok(())
    }

    //reads all the data on disk into a vec of u8
    //useful for reconstructing the map
    pub fn read_whole_disk(&mut self) -> Result<Vec<u8>, ()> {
        //buf to read result into
        let mut result_buf = Vec::new();

        let mut keep_reading = true;

        let mut block = 0;

        while keep_reading {
            //give the buf a place to read into
            result_buf.extend_from_slice(&[0; 512]);
            //read current block
            read(self.bus, self.drive, block as u32, &mut result_buf[block*512..(block+1)*512])?;
            //if block is not all zerso keep reading
            keep_reading = result_buf[block*512..(block*512)+1].iter().all(|&b| b != 0);
            block = block +1;
        }
        //remove zeros from the end of the result
        while let Some(&byte) = result_buf.last() {
            if byte == 0 {
                result_buf.pop();
            } else {
                break;
            }
        }
        Ok(result_buf)
    }

    //reads all the data on disk into a vec of u8
    //useful for reconstructing the map - doesn't remove empty bytes so can use 8 byte iterators
    pub fn read_whole_disk_leave_bytes(&mut self) -> Result<Vec<u8>, ()> {
        //buf to read result into
        let mut result_buf = Vec::new();

        let mut keep_reading = true;

        let mut block = 0;

        while keep_reading {
            //give the buf a place to read into
            result_buf.extend_from_slice(&[0; 512]);
            //read current block
            read(self.bus, self.drive, block as u32, &mut result_buf[block*512..(block+1)*512])?;
            //if block is not all zeros keep reading
            keep_reading = result_buf[block*512..(block*512)+1].iter().all(|&b| b != 0);
            block = block +1;
        }
        Ok(result_buf)
    }

    //appends to the end of the disk
    //useful for adding logs for changes to the map
    pub fn append_to_disk(&mut self, mut buf_to_write: Vec<u8>) -> Result<(), ()> {
        //get the data from the current block
        let mut result_buf = [0; 512].to_vec();
        read(self.bus, self.drive, self.current_block as u32, &mut result_buf[0..512])?;
        let current_block_data = result_buf[0..(self.current_address_in_block)].to_vec();
        //adds the data from the current block to the front of buf_to_write
        buf_to_write.splice(0..0,current_block_data);

        //then write just like over write disk but start at the current_block
        let length = buf_to_write.len();

        for i in 0..(length / 512) {
            let start_index = i * 512;
            let end_index = (i + 1) * 512;
            write(self.bus, self.drive, (i+self.current_block) as u32, &buf_to_write[start_index..end_index])?;
        }
        //save the current block and block adress so we can add logs
        self.current_block = self.current_block + (length / 512);
        self.current_address_in_block = 0;
        //if the log does not fit on 512 byte boundary need to make it so it does
        if length % 512 != 0 {
            let last_block_size = length % 512;
            let last_buf: Vec<u8> = buf_to_write[(length - last_block_size)..].to_vec();
            //write a whole block of 0 to signify end of data
            let mut padded_last_buf: Vec<u8> = [0; 512].to_vec();
            padded_last_buf[..last_block_size].copy_from_slice(&last_buf);
    
            write(self.bus, self.drive, self.current_block as u32, &padded_last_buf)?;
            self.current_address_in_block = self.current_address_in_block + length %512;
        }
        //write a whole block of 0 to signify end of data
        let empty_buf = [0; 512].to_vec();
        write(self.bus, self.drive, (self.current_block +1) as u32, &empty_buf)?;
        Ok(())
    }
}