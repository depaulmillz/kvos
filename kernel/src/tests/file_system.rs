extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use crate::tests::KernelTest;

use crate::disk::ata;
use crate::disk::ata::write;
use crate::disk::ata::read;
use crate::disk::ata::Drive;
//print benchmark and test results
use crate::serial_println;
//timing for benchmarks
use crate::drivers::timing;
//writing vec u8 
use crate::disk::disk_api::Disk;

//basic ATA test
fn test_read_write_disk(){
    let bus = 0;
    //use drive 1 for unit tests, don't write over the main disk storing important disk
    let drive = 1;
    //write to the second block of the disk
    let block = 1;
    ata::init();
    Drive::open(bus, drive);
    //the 512 bytes of dead beed to write
    let buf_dead_beef: &[u8] = "DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF".as_bytes();
    //write the dead beef
    let write_result: Result<(), ()> = write(bus,drive,block,buf_dead_beef);
    //make sure write went okay
    assert_eq!(write_result, Ok(()));
    //512 byte buffer to read into
    let buffer_read: &mut [u8] = &mut [0; 512];
    //read into empty buffer
    let read_result = read(bus,drive,block,buffer_read);
    //make sure read went okay
    assert_eq!(read_result, Ok(()));
    //make sure what was read equals what was written
    assert_eq!(buf_dead_beef, buffer_read);
    //write all 0x00 to disk to leave no trace of unit test
    let buffer_disk_clear: &mut [u8] = &mut [0; 512];
    let clear_result: Result<(), ()> = write(bus,drive,block,buffer_disk_clear);
    assert_eq!(clear_result, Ok(()));
}

//basic ATA test
fn test_disk_multiple_blocks(){
    let bus = 0;
    //use drive 1 for unit tests, don't write over the main disk storing important disk
    let drive = 1;
    ata::init();
    Drive::open(bus, drive);
    //from block 0 to 9 write the block number
    for block in 0..10{
        let buf_512_data: &[u8] = &[block + 48; 512];
        let buf_read: &mut [u8] = &mut [0; 512];
        let write_result: Result<(), ()> = write(bus,drive,block.into(),buf_512_data);
        //make sure write went okay
        assert_eq!(write_result, Ok(()));
        let read_result = read(bus,drive,block.into(),buf_read);
        //make sure read went okay
        assert_eq!(read_result, Ok(()));
        //make sure what was read equals what was written
        assert_eq!(buf_512_data, buf_read);
    }
    //from block 0 to 9 write 0x00 to the block to clear it
    for block in 0..10{
        let buf_512_data: &[u8] = &[0; 512];
        let buf_read: &mut [u8] = &mut [0; 512];
        let write_result: Result<(), ()> = write(bus,drive,block,buf_512_data);
        //make sure write went okay
        assert_eq!(write_result, Ok(()));
        let read_result = read(bus,drive,block,buf_read);
        //make sure read went okay
        assert_eq!(read_result, Ok(()));
        //make sure what was read equals what was written
        assert_eq!(buf_512_data, buf_read);
    }
}

//benckmarking the ata
//each test writes/reads 512 bytes 50 times which is 0.0256 MB
fn bench_ata(){
    ata::init();
    let bus = 0;
    let drive = 1;
    let block = 0;
    Drive::open(0, 1);
    let buf_512_data: &[u8] = &[0xFF; 512];
    let buf_read: &mut [u8] = &mut [0; 512];
    let mut start = timing::get_ticks();
    //0% Read
    for _i in 1..10 {
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
    }
    serial_println!("\n0% Read ran in {} ms",timing::get_ticks()-start);
    //20% Read
    start = timing::get_ticks();
    for _i in 1..10 {
        let _ = read(bus,drive,block,buf_read);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
    }
    serial_println!("20% Read ran in {} ms",timing::get_ticks()-start);
    //40% Read
    start = timing::get_ticks();
    for _i in 1..10 {
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
    }
    serial_println!("40% Read ran in {} ms",timing::get_ticks()-start);
    //60% Read
    start = timing::get_ticks();
    for _i in 1..10 {
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = write(bus,drive,block,buf_512_data);
        let _ = write(bus,drive,block,buf_512_data);
    }
    serial_println!("60% Read ran in {} ms",timing::get_ticks()-start);
    //80% Read
    start = timing::get_ticks();
    for _i in 1..10 {
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = write(bus,drive,block,buf_512_data);
    }
    serial_println!("80% Read ran in {} ms",timing::get_ticks()-start);
    //100% Read
    start = timing::get_ticks();
    for _i in 1..10 {
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
        let _ = read(bus,drive,block,buf_read);
    }
    serial_println!("100% Read ran in {} ms",timing::get_ticks()-start);
    //write all 0x00 to disk to leave no trace of unit test
    let buffer_disk_clear: &mut [u8] = &mut [0; 512];
    let clear_result: Result<(), ()> = write(bus,drive,block,buffer_disk_clear);
    assert_eq!(clear_result, Ok(()));
}
//Writing / Reading a vec to disk
fn disk_vec() {
    let bus = 0;
    let drive = 1;
    //make a new disk
    let mut disk = Disk::new(bus,drive);

    let mut big_buf: Vec<u8> = [58; 1500].to_vec();
    //write big buffer to disk
    let mut write_result = disk.over_write_disk(big_buf.clone());
    assert_eq!(write_result, Ok(()));
    //read all of disk
    let mut read_result = disk.read_whole_disk();
    //make sure what is read is what was written
    assert_eq!(big_buf, read_result.unwrap());
    //writing new buffer to disk
    big_buf = [78; 2300].to_vec();

    write_result = disk.over_write_disk(big_buf.clone());
    assert_eq!(write_result, Ok(()));

    read_result = disk.read_whole_disk();
    //ensure new buffer is what was read
    assert_eq!(big_buf, read_result.unwrap());
    //clear the disk
    big_buf = [0; 2300].to_vec();
    write_result = disk.over_write_disk(big_buf.clone());
    assert_eq!(write_result, Ok(()));

}

//Writing / Reading a vec to disk
fn log_to_disk() {
    let bus = 0;
    let drive = 1;
    //make new disk
    let mut disk = Disk::new(bus,drive);
    //first log
    let first_log: Vec<u8> = [60; 1500].to_vec();

    let mut write_result = disk.append_to_disk(first_log.clone());
    assert_eq!(write_result, Ok(()));

    let mut read_result = disk.read_whole_disk().unwrap();

    assert_eq!(first_log.len(), read_result.len());


    assert_eq!(first_log, read_result);
    //second log
    let mut second_log: Vec<u8> = [61; 2100].to_vec();

    write_result = disk.append_to_disk(second_log.clone());
    assert_eq!(write_result, Ok(()));

    read_result = disk.read_whole_disk().unwrap();

    assert_eq!(first_log.len()+second_log.len(), read_result.len());
    //make sure the result is the first log + the second log
    second_log.splice(0..0,first_log);
    assert_eq!(second_log, read_result);
    //third log
    let mut third_log: Vec<u8> = [62; 200].to_vec();

    write_result = disk.append_to_disk(third_log.clone());
    assert_eq!(write_result, Ok(()));

    read_result = disk.read_whole_disk().unwrap();
    //make sure the result is the first log + the second log + the third log
    third_log.splice(0..0,second_log);
    assert_eq!(third_log, read_result);
    //over write the disk
    let mut big_buf: Vec<u8> = [40; 600].to_vec();

    write_result = disk.over_write_disk(big_buf.clone());
    assert_eq!(write_result, Ok(()));

    read_result = disk.read_whole_disk().unwrap();
    //make sure the disk is what was overwritten
    assert_eq!(big_buf, read_result);
    //add a fourth log
    let mut fourth_log: Vec<u8> = [41; 1500].to_vec();

    write_result = disk.append_to_disk(fourth_log.clone());
    assert_eq!(write_result, Ok(()));

    read_result = disk.read_whole_disk().unwrap();
    //make sure fourth log is added to what was overwritten
    fourth_log.splice(0..0,big_buf);
    assert_eq!(fourth_log, read_result);

    //clear the disk
    big_buf = [0; 4000].to_vec();
    write_result = disk.over_write_disk(big_buf.clone());
    assert_eq!(write_result, Ok(()));

}
use crate::disk::persistentmap::{PersistentMap, PersistentHashMap};
//test insert into persistent map
fn test_persistent_map() {

    let map: PersistentHashMap<String, String> = PersistentHashMap::new();

    assert!(map.insert(&"key1".to_string(), &"value1".to_string()).unwrap());

    assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));

    assert!(map.insert(&"key2".to_string(), &"valueTWO".to_string()).unwrap());
    assert_eq!(map.get(&"key2".to_string()), Some("valueTWO".to_string()));
    //make new map from disk, make sure it gets the values from logs correctly
    let new_map: PersistentHashMap<String, String> = PersistentHashMap::build_from_disk();
    assert_eq!(new_map.get(&"key1".to_string()), Some("value1".to_string()));

    assert_eq!(new_map.get(&"key2".to_string()), Some("valueTWO".to_string()));
    assert!(new_map.remove(&"key1".to_string()).unwrap());
    assert_eq!(new_map.get(&"key1".to_string()), None);
}
//tests really long entires into the map
fn test_persistent_map_long_entries() {

    let map: PersistentHashMap<String, String> = PersistentHashMap::new();

    assert!(map.insert(&"keykeykeykeykeykeyONE".to_string(), &"value1value1value1value1value1value1value1value1".to_string()).unwrap());

    assert_eq!(map.get(&"keykeykeykeykeykeyONE".to_string()), Some("value1value1value1value1value1value1value1value1".to_string()));

    assert!(map.insert(&"key2".to_string(), &"valueTWO".to_string()).unwrap());
    assert_eq!(map.get(&"key2".to_string()), Some("valueTWO".to_string()));

    assert!(map.insert(&"keykeykeykeykeykeyThree".to_string(), &"value3value3".to_string()).unwrap());

    assert_eq!(map.get(&"keykeykeykeykeykeyThree".to_string()), Some("value3value3".to_string()));

    let new_map: PersistentHashMap<String, String> = PersistentHashMap::build_from_disk();
    assert_eq!(new_map.get(&"keykeykeykeykeykeyThree".to_string()), Some("value3value3".to_string()));
    assert_eq!(new_map.get(&"keykeykeykeykeykeyONE".to_string()), Some("value1value1value1value1value1value1value1value1".to_string()));

    assert_eq!(new_map.get(&"key2".to_string()), Some("valueTWO".to_string()));
}
//test remove log
fn test_persistent_map_remove() {

    let map: PersistentHashMap<String, String> = PersistentHashMap::new();

    assert!(map.insert(&"key1".to_string(), &"value1".to_string()).unwrap());

    assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));

    assert!(map.insert(&"key2".to_string(), &"valueTWO".to_string()).unwrap());
    assert_eq!(map.get(&"key2".to_string()), Some("valueTWO".to_string()));
    assert!(map.remove(&"key1".to_string()).unwrap());


    let new_map: PersistentHashMap<String, String> = PersistentHashMap::build_from_disk();
    assert_eq!(new_map.get(&"key1".to_string()), None);

    assert_eq!(new_map.get(&"key2".to_string()), Some("valueTWO".to_string()));
}
//test map compaction
fn test_persistent_map_compaction() {

    let mut map: PersistentHashMap<String, String> = PersistentHashMap::new();

    assert!(map.insert(&"key2".to_string(), &"valueTWO".to_string()).unwrap());
    assert_eq!(map.get(&"key2".to_string()), Some("valueTWO".to_string()));

    for _i in 0..100{
        assert!(map.insert(&"key1".to_string(), &"value1".to_string()).unwrap());
        assert!(map.remove(&"key1".to_string()).unwrap());

    }
    let bus = 0;
    let drive = 1;
    let mut disk = Disk::new(bus,drive);
    assert_eq!(disk.read_whole_disk().unwrap().len(),1020);
    assert_eq!(map.compact_logs(), Ok(()));
    assert_eq!(disk.read_whole_disk().unwrap().len(),5*8);
}


pub fn run_tests() {
    let tests = [
        KernelTest {
            name : "test_read_write_disk",
            test_fn : test_read_write_disk,
        },
        KernelTest {
            name : "test_disk_multiple_blocks",
            test_fn : test_disk_multiple_blocks,
        },
        KernelTest {
            name : "test_bench_ata",
            test_fn : bench_ata,
        },
        KernelTest {
            name : "test_disk_vec",
            test_fn : disk_vec,
        },
        KernelTest {
            name : "test_log_to_disk",
            test_fn : log_to_disk,
        },
        KernelTest {
            name : "test_persistent_map",
            test_fn : test_persistent_map,
        },
        KernelTest {
            name : "test_persistent_map_long_entries",
            test_fn : test_persistent_map_long_entries,
        },
        KernelTest {
            name : "test_persistent_map_remove",
            test_fn : test_persistent_map_remove,
        },
        KernelTest {
            name : "test_persistent_map_compaction",
            test_fn : test_persistent_map_compaction,
        },
    ];
    for t in tests.iter() {
        serial_print!("{}...\t", t.name);
        (t.test_fn)();
        serial_print!("[ok]\n");
    }
}
