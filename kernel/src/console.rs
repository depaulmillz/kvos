use alloc::string::String;
use spin::Mutex;
use x86_64::instructions::interrupts;





pub static STDIN: Mutex<String> = Mutex::new(String::new());

pub fn key_handle(key: char) {
    let mut stdin = STDIN.lock();
    let key = if (key as u32) < 0xFF { (key as u8) as char } else { key };
    stdin.push(key);
}

pub fn read_char() -> char {
    loop {
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            if !stdin.is_empty() {
                Some(stdin.remove(0))
            } else {
                None
            }
        });
        if let Some(c) = res {
            return c;
        }
    }
}

pub fn read_line() -> String {
    loop {
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            match stdin.chars().next_back() {
                Some('\n') => {
                    let line = stdin.clone();
                    stdin.clear();
                    Some(line)
                }
                _ => {
                    let line = stdin.clone();
                    Some(line)
                }
            }
        });
        if let Some(line) = res {
            return line;
        }
    }
}