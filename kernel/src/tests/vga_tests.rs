use super::KernelTest;
use crate::serial_print;

//// Tests
fn test_println_simple() {
    println!("test_println_simple output");
}

fn test_println_many() {
    for i in 0..200 {
        println!("test_print_many iter {}", i);
    }
}

pub fn run_tests() {
    let tests = [
        KernelTest {
            name : "test_println_simple",
            test_fn : test_println_simple,
        },
        KernelTest {
            name : "test_println_many",
            test_fn : test_println_many,
        }
    ];
    for t in tests.iter() {
        serial_print!("{}...\t", t.name);
        (t.test_fn)();
        serial_print!("[ok]\n");
    }
}

//fn test_println_output() {
//    use x86_64::instructions::interrupts;
//    use core::fmt::Write;
//
//    let s = "Some string";
//    assert!(s.len() <= BUFFER_WIDTH);
//
//    interrupts::without_interrupts(|| {
//        let mut writer = WRITER.lock();
//        writeln!(writer, "\n{}", s).expect("writeln! failed");
//        for (i, c) in s.chars().enumerate() {
//            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
//            assert_eq!(char::from(screen_char.ascii_character), c);
//        }
//    });
//}
//inventory::submit!(Test {
//    name : "test_println_output",
//    test_fn : test_println_output,
//});
