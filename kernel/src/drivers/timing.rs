use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts;
use asm::rdtsc;
use core::cmp::max;

pub const PIT_FREQUENCY  : f64 = 3_579_545.0 / 3.0;
const PIT_DIVIDER : usize = 1193;
const PIT_INTERVAL : f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS : AtomicU64 = AtomicU64::new(0);
static NANOSECONDS_PER_TICK : AtomicU64 = AtomicU64::new(0);

pub fn set_pit_frequency(divider : u16, channel : u8) {
    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40 + channel as u16);
        let operating_mode = 6; // Square wave generator
        let access_mode = 3; // Lobyte + Hibyte
        unsafe {
            cmd.write((channel << 6) | (access_mode << 4) | operating_mode);
            data.write(bytes[0]);
            data.write(bytes[1]);
        }
    });
}

pub fn get_ticks() -> u64 {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn add_tick() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Requires interrupts to be enabled
pub fn sleep(ms : f64) {

    assert!(interrupts::are_enabled());

    let start = PIT_INTERVAL * (get_ticks() as f64);
    let mut end = PIT_INTERVAL * (get_ticks() as f64);

    while (end - start) * 1e3 < ms {
        hlt();
        end = PIT_INTERVAL * (get_ticks() as f64);
    }
}

pub fn nanosleep(ns : u64) {
    let start = rdtsc();
    let mut end = rdtsc();

    let factor = NANOSECONDS_PER_TICK.load(Ordering::Relaxed);

    while (end - start) * factor < ns {
        end = rdtsc(); 
    }

}

pub fn init() {
    set_pit_frequency(PIT_DIVIDER as u16, 0);
}

pub fn init_nanosleep() {
    let start = rdtsc();
    
    let start_pit = PIT_INTERVAL * (get_ticks() as f64);
    let mut end_pit = PIT_INTERVAL * (get_ticks() as f64);

    while (end_pit - start_pit) * 1e3 < 1.0 {
        hlt();
        end_pit = PIT_INTERVAL * (get_ticks() as f64);
    }

    let end = rdtsc();
    let ns = (end_pit - start_pit) * 1e9 as f64;

    NANOSECONDS_PER_TICK.store(max((ns / ((end - start) as f64)) as u64, 1), Ordering::SeqCst);
}
