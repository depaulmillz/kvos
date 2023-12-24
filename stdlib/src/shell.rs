use crate::syscall;
use crate::print;
use crate::println;

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use syscall::{read_kv, write_kv, delete_kv, read_in};

fn input(prompt: String) -> String {
    print!("{}", prompt);
    let mut buf = [0u8; 256];
    let mut len;
    let mut old_len = 0;
    loop {
        len = read_in(&mut buf, 256);
        if len > old_len {
            //check if the last char is \n
            if buf[len-1] == 10 {
                println!();
                return String::from_utf8(buf[0..len-1].to_vec()).unwrap();
            }
            let new = String::from_utf8(buf[old_len..len].to_vec()).unwrap();
            print!("{}", new);
        }
        old_len = len;
    }
}


pub fn shell() {
    loop {
        let input = input("user@kvos: ".to_string());
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["read_kv", key] => {
                let values: Vec<String> = read_kv(vec![key.to_string()]);
                let val = values.get(0).unwrap();
                println!("{}: {}", key, val)
            },
            ["write_kv", key, val] => {
                write_kv(vec![key.to_string()], vec![val.to_string()]);
                println!("Value written");
            },
            ["delete_kv", key] => {
                delete_kv(vec![key.to_string()]);
                println!("Value deleted");
            }
            ["echo", val] => {
                println!("{}", val);
            },
            ["exit"] => {
                println!("Exiting...");
                break;
            },
            ["help"] => {
                println!("Commands:");
                println!("read_kv <key>");
                println!("write_kv <key> <value>");
                println!("exit");
            },
            _ => println!("Unknown command"),
        }
    }
}
