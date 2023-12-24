//!
//! Serially print to qemu
//!
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// Serial port for writing to qemu stdout
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts( || {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}

/// Print formatted string to serial in qemu
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

/// Print formatted string as a line to serial in qemu
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
            concat!($fmt, "\n"), $($arg)*));
}

/// Print formatted trace in qemu
#[macro_export]
macro_rules! serial_traceln {
    () => ($crate::serial_println!("[TRACE: {}:{}]\t", file!(), line!()));

    ($fmt:expr) => ($crate::serial_println!(concat!("[TRACE: {}:{}]\t", $fmt), file!(), line!()));

    ($fmt:expr, $($arg:tt)*) => ($crate::serial_println!(
            concat!("[TRACE: {}:{}]\t", $fmt), file!(), line!(),
            $($arg)*));
}

/// Print formatted info in qemu
#[macro_export]
macro_rules! serial_infoln {
    () => ($crate::serial_println!("[INFO: {}:{}]\t", file!(), line!()));

    ($fmt:expr) => ($crate::serial_println!(concat!("[INFO: {}:{}]\t", $fmt), file!(), line!()));

    ($fmt:expr, $($arg:tt)*) => ($crate::serial_println!(
            concat!("[INFO: {}:{}]\t", $fmt), file!(), line!(),
            $($arg)*));
}

/// Print formatted warning in qemu
#[macro_export]
macro_rules! serial_warnln {
    () => ($crate::serial_println!("[WARN: {}:{}]\t", file!(), line!()));

    ($fmt:expr) => ($crate::serial_println!(concat!("[WARN: {}:{}]\t", $fmt), file!(), line!()));

    ($fmt:expr, $($arg:tt)*) => ($crate::serial_println!(
            concat!("[WARN: {}:{}]\t", $fmt), file!(), line!(),
            $($arg)*));
}

/// Print formatted error in qemu
#[macro_export]
macro_rules! serial_errorln {
    () => ($crate::serial_println!("[ERROR: {}:{}]\t", file!(), line!()));

    ($fmt:expr) => ($crate::serial_println!(concat!("[ERROR: {}:{}]\t", $fmt), file!(), line!()));

    ($fmt:expr, $($arg:tt)*) => ($crate::serial_println!(
            concat!("[ERROR: {}:{}]\t", $fmt), file!(), line!(),
            $($arg)*));
}

/// Print formatted debug statement in qemu
#[macro_export]
macro_rules! serial_debugln {
    () => ($crate::serial_println!("[DEBUG: {}:{}]\t", file!(), line!()));

    ($fmt:expr) => ($crate::serial_println!(concat!("[DEBUG: {}:{}]\t", $fmt), file!(), line!()));

    ($fmt:expr, $($arg:tt)*) => ($crate::serial_println!(
            concat!("[DEBUG: {}:{}]\t", $fmt), file!(), line!(),
            $($arg)*));
}
