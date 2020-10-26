use clap::{Arg, App};
mod emu;

fn main() {
    let matches = App::new("oemu")
    .version("0.1.0")
    .author("Trevor McKay <trmckay@calpoly.edu>")
    .about("Emulator for the RV32I multi-cycle Otter")
    .arg("-d, --debug         'Emulates the MCU in debug mode'")
    .arg("-b, --binary=[FILE] 'Runs the specified binary'")
    .arg("-k, --break=[PC]    'Adds a breakpoint at the specified program counter'")
    .get_matches();

    let debug = match matches.occurrences_of("debug") {
        0 => false,
        _ => true
    };

    let mut bps: Vec<u32> = Vec::new();
    match matches.values_of("break") {
        None => (),
        Some(values) => {
            for bp in values {
                match u32::from_str_radix(bp.trim_start_matches("0x"), 16) {
                    Err(why) => panic!("Error: could not parse breakpoint '{}': {}", bp, why),
                    Ok(pc) => bps.push(pc)
                };
            }
        }
    }

    let bin;
    if let Some(b) = matches.value_of("binary") {
        bin = String::from(b);
    }
    else {
        println!("Warning: no binary file specified, using default mem.bin");
        bin = String::from("mem.bin");
    }

    let mut opts = emu::Options {
        bin: bin,
        debug: debug,
        bps: bps,
        log_path: String::from("oemu.log"),
        log_to_f: true
    };

    println!("binary: ./{}", opts.bin);
    println!("debug: {}", opts.debug);
    println!("breakpoints: {:?}", opts.bps);

    let mut line = String::new();
    println!("Press enter to begin");
    std::io::stdin().read_line(&mut line).unwrap();

    emu::emulate(&mut opts);
}