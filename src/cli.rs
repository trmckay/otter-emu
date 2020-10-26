use super::*;

pub fn refresh_ui(mcu: &otter::MCU, ir: &(otter::rv32i::Instruction, u32), opts: &Options) {
        print!("\x1B[2J\x1B[1;1H"); // clear terminal

        println!("+---------------------------------------------------+");
        println!("| PC: {:#010X}", mcu.pc);

        if opts.debug {
            println!("+---------------------------------------------------+");
            println!("|      Instruction: {:?} ({:#010X})",
                ir.0.op, ir.1);
            println!("|  Operand 1 (rs1): x{:#} = {:#} / {:#x}",
                ir.0.rs1, mcu.rf.rd(ir.0.rs1), mcu.rf.rd(ir.0.rs1));
            println!("|  Operand 2 (rs2): x{:#} = {:#} / {:#x}",
                ir.0.rs2, mcu.rf.rd(ir.0.rs2), mcu.rf.rd(ir.0.rs2));
            println!("| Destination (rd): {:#}", ir.0.rd);
            println!("|        Immediate: {:#} / {:#x}", ir.0.imm, ir.0.imm);

            println!("+---------------------------------------------------+");

            println!(  "|   Register |     Signed |   Unsigned |        Hex");
            for i in 0..32 {
                let name = RF_NAMES[i];
                let data = mcu.rf.rd(i as u32);
                print!("| ");
                if i < 10 {
                    print!(" ")
                }
                print!("x{}", i);

                println!(", {}  | {:#10} | {:#10} | {:#10X}", name, data as i32, data, data);
            }
        }

        println!("+---------------------------------------------------+");

        println!("|            F E D C B A 9 8 7 6 5 4 3 2 1 0");
        print!("|      LEDs: ");
        for led in mcu.leds().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }

        print!("\n|  Switches: ");
        for led in mcu.switches().iter().rev() {
            if *led {
                print!("* ");
            }
            else {
                print!("- ");
            }
        }
        println!("\n| 7-Segment: {:#06X}", mcu.sseg());

        println!("+---------------------------------------------------+");
}