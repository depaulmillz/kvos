use alloc::string::{String, ToString};

use crate::kvstore::KVStore;
use crate::cc::Transaction;
use crate::println;
use crate::KVSTORE;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;


pub struct Random {
    rng:  XorShiftRng,
    seed: [u8; 16],
}

impl Random {
    pub fn new() -> Self {
        let mut seed = [0u8; 16];
        for i in 0..16 {
            seed[i] = i as u8;
        }
        let rng = XorShiftRng::from_seed(seed);
        Random {
            rng,
            seed,
        }
    }

    pub fn get_random(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min + 1;
        let bias = (u64::MAX % range + 1) % range;
    
        loop {
            let random_value = self.rng.next_u64();
            if random_value >= bias {
                return min + (random_value - bias) % range;
            }
        }
    }

    pub fn get_sead(&self) -> [u8; 16] {
        self.seed
    }

    pub fn set_seed(&mut self, seed: [u8; 16]) {
        self.seed = seed;
        self.rng = XorShiftRng::from_seed(seed);
    }
}


pub fn populate(max_key: i32, pop:i32, rng: &mut Random) {
    for _ in 0..pop {
        let key = rng.get_random(0, max_key as u64).to_string();
        KVSTORE.transact(|tx| {
            tx.write(&key, &String::from("value"));
        }, false);
    }
}

pub fn benchmark(max_key: i32, 
                 iters: i32, 
                 ratio: u64, 
                 rng: &mut Random) -> f64 {


    let start = asm::rdtsc();
    
    for _ in 1..iters {
        let key = rng.get_random(0, max_key as u64).to_string();
        let op  = rng.get_random(0, 100 as u64);

        if op <= ratio {
            //read
            KVSTORE.transact(|tx| {
                tx.read(&key);
            }, false);
        } else if op <= ratio + (100-ratio) / 2 {
            //remove
            KVSTORE.transact(|tx| {
                tx.delete(&key);
            }, false);
        } else {
            //insert
            KVSTORE.transact(|tx| {
                tx.write(&key, &String::from("value"));
            }, false);
        }
    }
    let end = asm::rdtsc();
    return (end - start) as f64 / iters as f64;
}

pub fn clear_kvstore(max_key: i32) {
    KVSTORE.transact(|tx| {
        for key in 0..max_key {
            tx.delete(&key.to_string());
        }
    }, false);
}


pub fn run_bench() {
    let mut rng = Random::new();
    let max_key = 2000;
    let pop = 1000;
    let iters = 1000;
    let ratio = 80;
    populate(max_key, pop, &mut rng);
    let time = benchmark(max_key, iters, ratio, &mut rng);
    println!("Time per operation: {}", time);
    clear_kvstore(max_key);
}
