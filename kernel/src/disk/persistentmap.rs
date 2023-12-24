extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::mutex::Mutex;
use crate::common::hash::Mix13Hash;
use crate::disk::disk_api::Disk;
pub trait PersistentMap {
    type Key;
    type Value;
    fn new() -> Self;
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn insert(&self, key: &Self::Key, value: &Self::Value) -> Result<bool,()>;
    fn remove(&self, key: &Self::Key) -> Result<bool,()>;
    fn build_from_disk() -> Self;
    fn insert_no_log(&self, key: &Self::Key, value: &Self::Value) -> bool;
    fn remove_no_log(&self, key: &Self::Key) -> bool;
    fn compact_logs(&mut self) -> Result<(),()>;
}
//trait that allows a type to be converted to/from a byte array
pub trait ToBeBytes {
    type ByteArray: AsRef<[u8]>;
    fn to_be_bytes(&self) -> Self::ByteArray;
    //returns the length as a byte array representing a u64
    fn len(&self) -> Vec<u8> {
        let len_bytes = (self.to_be_bytes().as_ref().len() as u64).to_be_bytes();
        len_bytes.to_vec()
    }

    fn to_vec(&self) -> Vec<u8> {
        self.to_be_bytes().as_ref().to_vec()
    }

    fn from_vec(bytes: Vec<u8>) -> Self;

}

impl ToBeBytes for String {
    type ByteArray = Vec<u8>;
    
    fn to_be_bytes(&self) -> Self::ByteArray {
        let bytes = self.as_bytes().to_vec();
        bytes
    }

    fn from_vec(bytes: Vec<u8>) -> Self {
        String::from_utf8_lossy(&bytes).to_string()
    }
}

struct Entry<K, V> {
    key: K,
    value: V,
}

struct Bucket<K, V> {
    data: Mutex<Vec<Entry<K, V>>>,
}

pub struct PersistentHashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    num_buckets: usize,
    disk: Mutex<Disk>,
}
impl <K: Eq + core::hash::Hash + AsRef<[u8]>, V: Clone>  PersistentHashMap<K, V> {
    pub fn with_capacity(num_buckets:usize) -> Self {
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Mutex::new(Vec::new()),
            });
        }
        let disk = Disk::new(0,1);

        PersistentHashMap { buckets, num_buckets,disk: Mutex::new(disk) }
    } 
}
impl<K: Eq + Clone + core::hash::Hash + AsRef<[u8]> + ToBeBytes, V: Clone + ToBeBytes> PersistentMap for PersistentHashMap<K, V> {
    type Key = K;
    type Value = V;

    fn new() -> Self {
        let num_buckets = 16; // Default number of buckets
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Mutex::new(Vec::new()),
            });
        }
        let disk = Disk::new(0,1);
        PersistentHashMap { buckets, num_buckets,disk: Mutex::new(disk), }
    }

    //builds a map from what is on disk
    fn build_from_disk() -> Self {
        let num_buckets = 16; // Default number of buckets
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Bucket {
                data: Mutex::new(Vec::new()),
            });
        }
        //makes a disk for the map
        let disk = Disk::new(0,1);
        let resulting_map =PersistentHashMap { buckets, num_buckets,disk: Mutex::new(disk) };
        //get the logs from disk
        let mut disk_data:Vec<Vec<u8>> = resulting_map.disk.lock().read_whole_disk_leave_bytes().unwrap().chunks_exact(8).map(|chunk| chunk.to_vec()).collect();
        //iterate over the logs in 8 byte chunks
        let mut data_iter = disk_data.iter_mut();
        while let Some(entry) = data_iter.next() {
            match  String::from_utf8(entry.to_vec()).unwrap().as_str(){
                "KVKVKVKV" => {
                    if let Some(key_length_bytes) = data_iter.next() {
                        //get the key length
                        let key_length: u64 = u64::from_be_bytes([
                            key_length_bytes[0], key_length_bytes[1], key_length_bytes[2], key_length_bytes[3],
                            key_length_bytes[4], key_length_bytes[5], key_length_bytes[6], key_length_bytes[7],
                        ]);
                        //construct the key
                        let mut key_value:Vec<u8> = Vec::with_capacity(key_length as usize);
                        for _i in 0..(key_length/8){
                            if let Some(chunk) = data_iter.next(){
                                key_value.append(chunk);
                            }
                        }
                        if key_length % 8 != 0 {
                            if let Some(chunk) = data_iter.next(){
                                key_value.append(chunk);
                            }
                        }
                        //remove zeros from the end of the result
                        while  key_value.len() > key_length as usize{
                            key_value.pop();
                        }
                        let key: K = ToBeBytes::from_vec(key_value);
                        //get the value
                        if let Some(value_length_bytes) = data_iter.next() {
                            //get value length
                            let value_length: u64 = u64::from_be_bytes([
                                value_length_bytes[0], value_length_bytes[1], value_length_bytes[2], value_length_bytes[3],
                                value_length_bytes[4], value_length_bytes[5], value_length_bytes[6], value_length_bytes[7],
                            ]);
                            //construct the value
                            let mut value_value:Vec<u8> = Vec::with_capacity(value_length as usize);
                            
                            for _i in 0..(value_length/8){
                                if let Some(chunk) = data_iter.next(){
                                    value_value.append(chunk);
                                }
                                
                            }
                            if value_length % 8 != 0 {
                                if let Some(chunk) = data_iter.next(){
                                    value_value.append(chunk);
                                }
                            }

                            //remove zeros from the end of the result
                            while  value_value.len() > value_length as usize{
                                value_value.pop();
                            }
                            let value: V = ToBeBytes::from_vec(value_value);
                            
                            resulting_map.insert_no_log(&key,&value);
                        }
                    }
                    
                    
                },
                "KVREMOVE" => {
                    if let Some(key_length_bytes) = data_iter.next() {
                        //get key length
                        let key_length: u64 = u64::from_be_bytes([
                            key_length_bytes[0], key_length_bytes[1], key_length_bytes[2], key_length_bytes[3],
                            key_length_bytes[4], key_length_bytes[5], key_length_bytes[6], key_length_bytes[7],
                        ]);
                        let mut key_value:Vec<u8> = Vec::with_capacity(key_length as usize);
                        //construct key
                        for _i in 0..(key_length/8){
                            if let Some(chunk) = data_iter.next(){
                                key_value.append(chunk);
                            }
                        }
                        if key_length % 8 != 0 {
                            if let Some(chunk) = data_iter.next(){
                                key_value.append(chunk);
                            }
                        }
                        //remove zeros from the end of the result
                        while  key_value.len() > key_length as usize{
                            key_value.pop();
                        }
                        let key: K = ToBeBytes::from_vec(key_value);
                        resulting_map.remove_no_log(&key,);
                    }
                },
                //all logs processed
                _ => break,
            }
        }
        //return the new map built from the logs
        resulting_map
    }

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        let bucket_idx = self.compute_bucket_idx(key);
        self.buckets[bucket_idx].data.lock().iter().find(|e| &e.key == key).map(|e| e.value.clone())
    }

    fn insert_no_log(&self, key: &Self::Key, value: &Self::Value) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &self.buckets[bucket_idx];
        let mut bucket_data = bucket.data.lock();
        if let Some(entry) = bucket_data.iter_mut().find(|e| &e.key == key) {
            entry.value = value.clone();
            true
        } else {
            bucket_data.push(Entry {
                key: key.clone(),
                value: value.clone(),
            });
            true
        }
    }
    fn insert(&self, key: &Self::Key, value: &Self::Value) ->Result<bool,()> {

        let bucket_idx = self.compute_bucket_idx(key);
        
        let bucket = &self.buckets[bucket_idx];

        let mut bucket_data = bucket.data.lock();
        
        //logging KV INSERT
        let mut request : Vec<u8> = "KVKVKVKV".as_bytes().to_vec();
        
        request.append(&mut key.len());
        request.append(&mut key.to_vec());
        while request.len()%8 != 0{
            request.push(0);
        }
        
        request.append(&mut value.len());
        request.append(&mut value.to_vec());
        while request.len()%8 != 0{
            request.push(0);
        }
        //write log to disk
        self.disk.lock().append_to_disk(request)?;

        if let Some(entry) = bucket_data.iter_mut().find(|e| &e.key == key) {
            entry.value = value.clone();
            Ok(true)
        } else {
            bucket_data.push(Entry {
                key: key.clone(),
                value: value.clone(),
            });
            Ok(true)
        }
    }

    fn remove_no_log(&self, key: &Self::Key) -> bool {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &self.buckets[bucket_idx];
        let mut bucket_data = bucket.data.lock();
        if let Some(pos) = bucket_data.iter().position(|e| &e.key == key) {
            bucket_data.remove(pos);
            true
        } else {
            false
        }
    }

    fn remove(&self, key: &Self::Key) -> Result<bool,()> {
        let bucket_idx = self.compute_bucket_idx(key);
        let bucket = &self.buckets[bucket_idx];
        let mut bucket_data = bucket.data.lock();
        if let Some(pos) = bucket_data.iter().position(|e| &e.key == key) {
            //logging KV INSERT
            let mut request : Vec<u8> = "KVREMOVE".as_bytes().to_vec();
            request.append(&mut key.len());
            request.append(&mut key.to_vec());
            while request.len()%8 != 0{
                request.push(0);
            }
            self.disk.lock().append_to_disk(request)?;
            //Actually do the remove
            bucket_data.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn compact_logs(&mut self) -> Result<(),()>{
        let mut request : Vec<u8> = Vec::new();
        for bucket in self.buckets.iter(){
            for entry in bucket.data.lock().iter(){
                request.append(&mut "KVKVKVKV".as_bytes().to_vec());
                //add key to vec
                request.append(&mut entry.key.len());
                request.append(&mut entry.key.to_vec());
                while request.len()%8 != 0{
                    request.push(0);
                }
                //add value to vec
                request.append(&mut entry.value.len());
                request.append(&mut entry.value.to_vec());
                while request.len()%8 != 0{
                    request.push(0);
                }
            }
        }
        self.disk.lock().over_write_disk(request)
    }

}

impl<K: Eq + core::hash::Hash + AsRef<[u8]>, V> PersistentHashMap<K, V> {
    fn compute_bucket_idx(&self, key: &K) -> usize {
        let hash = Mix13Hash::new().compute_hash(&key.as_ref().to_vec());
        (hash % self.num_buckets as u64) as usize
    }
}
