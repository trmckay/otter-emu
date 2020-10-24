#[path = "./otter.rs"] mod otter;
use std::time;

fn run_cli(mcu: &mut otter::MCU) {

    let paused = false;
    let mut from: u32;

    loop {
        let t0 = time::Instant::now();

        // save current pc before executing, useful for debugging jumps/branches
        from = mcu.pc;

        // exec 1
        if !paused {
            mcu.step();
        }

        print!("\x1B[2J\x1B[1;1H"); // clear terminal
        println!("          F E D C B A 9 8 7 6 5 4 3 2 1 0");
        print!("    leds: ");
        for led in mcu.leds().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }
        print!("\nswitches: ");
        for led in mcu.switches().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }
        println!("");
        println!("    sseg: {:#04x}", mcu.sseg());
        println!("      pc: {:#08x}", mcu.pc);
        println!("    from: {:#08x}", from);

        let t1 = time::Instant::now();
        let dt = (t1 - t0).as_micros();
        let clk = (1 as f64)/(dt as f64) * 2000 as f64;
        println!(" eqv clk: {:#.02} kHz", clk);
        println!("{}", 0b1011 as i32);
    }
}

pub fn emulate(bin: &str) {
    let mut mcu = otter::MCU::new();
    mcu.load_bin(bin);
    run_cli(&mut mcu);
}