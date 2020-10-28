use super::*;

pub fn run_cli(mcu: &mut otter::MCU, opts: &mut Options) {

    let mut last_pc = 0;
    loop {

        thread::sleep(time::Duration::from_millis(2));

        let ir = mcu.fetch();
        cli::refresh_ui(&mcu, &ir, opts);

        if opts.bps.contains(&mcu.pc) && !opts.debug {
            opts.debug = true;
            cli::refresh_ui(&mcu, &ir, opts);
            println!("\nHit breakpoint {:#010X}, came from {:#010X}\nPress enter to step", mcu.pc, last_pc);
        }

        if opts.debug {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            match line[..].trim_end_matches("\n") {
                ":c" => opts.debug = false,
                ":m" => {
                    print!("addr: ");
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                    match u32::from_str_radix(line.trim_start_matches("0x").trim_end_matches("\n"), 16) {
                        Ok(val) => (println!("mem[{:#010X}] = {:#010X}",
                            val, mcu.mem.rd(val, otter::mem::Size::Word))),
                        Err(why) => eprintln!("Error: not a valid address: {}", why)
                    }
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                }
                ":q" => return,
                _ => ()
            }
        }
        last_pc = mcu.pc;
        mcu.exec(ir.0);
    }
}

pub fn refresh_ui(mcu: &otter::MCU, ir: &(otter::rv32i::Instruction, u32), opts: &Options) {
        print!("\x1B[2J\x1B[1;1H"); // clear terminal

        println!("+----------------------------------------------------+");
        println!("| PC: {:#010X}", mcu.pc);

        if opts.debug {
            println!("+----------------------------------------------------+");
            println!("|      Instruction: {:?} ({:#010X})",
                ir.0.op, ir.1);
            println!("|  Operand 1 (rs1): x{:#} = {:#} / {:#x}",
                ir.0.rs1, mcu.rf.rd(ir.0.rs1), mcu.rf.rd(ir.0.rs1));
            println!("|  Operand 2 (rs2): x{:#} = {:#} / {:#x}",
                ir.0.rs2, mcu.rf.rd(ir.0.rs2), mcu.rf.rd(ir.0.rs2));
            println!("| Destination (rd): {:#}", ir.0.rd);
            println!("|        Immediate: {:#} / {:#x}", ir.0.imm, ir.0.imm);

            println!("+----------------------------------------------------+");

            println!(  "|   Register |      Signed |    Unsigned |        Hex");
            println!(  "|------------|-------------|-------------|------------");
            for i in 0..32 {
                let name = RF_NAMES[i];
                let data = mcu.rf.rd(i as u32);
                print!("| ");
                if i < 10 {
                    print!(" ")
                }
                print!("x{}", i);

                println!(", {}  | {:#11} | {:#11} | {:#10X}", name, data as i32, data, data);
            }
        }

        println!("+----------------------------------------------------+");

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

        println!("+----------------------------------------------------+");
}