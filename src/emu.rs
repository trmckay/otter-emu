#[path = "./otter.rs"] mod otter;
use std::{thread, time};

fn refresh_ui(mcu: &otter::MCU, debug: bool) {
        print!("\x1B[2J\x1B[1;1H"); // clear terminal

        println!(" PC: {:#08x}", mcu.pc);
        println!("+-------------------------------------------+");
        if debug {
            let ir = mcu.fetch();
            println!("      Instruction: {:?} ({:#08x})",
                ir.op, mcu.mem.rd(mcu.pc, otter::mem::Size::Word));
            println!("  Operand 1 (rs1): x{:#} = {:#} / {:#x}",
                ir.rs1, mcu.rf.rd(ir.rs1), mcu.rf.rd(ir.rs1));
            println!("  Operand 2 (rs2): x{:#} = {:#} / {:#x}",
                ir.rs2, mcu.rf.rd(ir.rs2), mcu.rf.rd(ir.rs2));
            println!(" Destination (rd): {:#}", ir.rd);
            println!("        Immediate: {:#} / {:#x}", ir.imm, ir.imm);

            println!("+-------------------------------------------+");

            println!(" Registers:");
            for i in 0..32 {
                let rd = mcu.rf.rd(i);
                println!("   x{:#02} = {:#} / {:#08x}", i, rd, rd);
            }
        }

        println!("+-------------------------------------------+");

        println!("           F E D C B A 9 8 7 6 5 4 3 2 1 0");
        print!("      LEDs: ");
        for led in mcu.leds().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }

        print!("\n  Switches: ");
        for led in mcu.switches().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }
        println!("");
        println!(" 7-Segment: {:#04x}", mcu.sseg());

        println!("+-------------------------------------------+");


}

fn run_cli(mcu: &mut otter::MCU) {
    let debug = unsafe { super::DEBUG };

    loop {
        refresh_ui(mcu, debug);

        if debug {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
        }

        // exec 1
        mcu.step();
        thread::sleep(time::Duration::from_millis(2));
    }
}

pub fn emulate(bin: &str) {
    let mut mcu = otter::MCU::new();
    mcu.load_bin(bin);
    run_cli(&mut mcu);
}