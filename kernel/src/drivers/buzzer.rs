use super::timing;
use x86_64::instructions::port::Port;

pub fn buzz(freq : f64, time : f64) {
    timing::set_pit_frequency((timing::PIT_FREQUENCY / freq) as u16, 2);
    let mut speaker : Port<u8> = Port::new(0x61);
    let tmp = unsafe { speaker.read() };
    if tmp != (tmp | 3) {
        unsafe { speaker.write(tmp | 3) } ;
    }

    timing::sleep(time);
    unsafe { 
        let tmp = speaker.read();
        speaker.write( tmp & 0xFC ); 
    timing::sleep(time);
    };
    timing::sleep(1.0);
}

pub fn songs() {

    println!("Playing boot up song!");

    let note = 800.0;

    buzz(659., note);
    buzz(659., note);
    buzz(659., note * 2.0);

    buzz(659., note);
    buzz(659., note);
    buzz(659., note * 2.0);

    buzz(659., note);
    buzz(783., note);
    buzz(523., note);
    buzz(587., note);
    buzz(659., note * 2.0);
    
    timing::sleep(1000.0);

    let bflat5 = 932.33;
    let eflat5 = 622.25;
    let aflat4 = 415.30;
    let g5 = 783.99;

    // b flat
    buzz(bflat5, note * 3.0 / 4.0);
    // e flat
    buzz(eflat5, note / 2.0);
    // a flat
    buzz(aflat4, note);
    // g
    buzz(g5, note * 3.0 / 2.0);
    // b flat
    buzz(bflat5, note / 4.0);
    // a flat
    buzz(aflat4, note * 2.0);

    println!("Done with boot up song!");
}

