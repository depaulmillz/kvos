use alloc::string::{String, ToString};
use alloc::vec;

use crate::println;
use crate::rand::Random;
use crate::syscall::{write_kv, read_kv, delete_kv};


pub fn populate(max_key: i32, pop:i32, rng: &mut Random) {
    for _ in 0..pop {
        let key = rng.get_random(0, max_key as u64).to_string();
        let key_vec = vec![key];
        let val_vac = vec![String::from("value")];
        write_kv(key_vec, val_vac);
    }
}

pub fn benchmark(max_key: i32, 
                 iters: i32, 
                 ratio: u64, 
                 rng: &mut Random) -> f64 {


    let start = asm::rdtsc();
    
    for _ in 1..iters {
        let key = rng.get_random(0, max_key as u64).to_string();
        let key_vec = vec![key];
        let op  = rng.get_random(0, 100 as u64);

        if op <= ratio {
            //read
            read_kv(key_vec);
        } else if op <= ratio + (100-ratio) / 2 {
            //remove
            delete_kv(key_vec)
        } else {
            //insert
            let val_vac = vec![String::from("value")];
            write_kv(key_vec, val_vac);
        }
    }
    let end = asm::rdtsc();
    return (end - start) as f64 / iters as f64;
}

fn clear_kvstore(max_key: i32) {
    for key in 0..max_key {
        let key_vec = vec![key.to_string()];
        delete_kv(key_vec);
    }
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
