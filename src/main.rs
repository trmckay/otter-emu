use clap::{Arg, App};
mod emu;

static mut DEBUG: bool = false;

fn main() {
    let matches = App::new("oemu")
    .version("0.1.0")
    .author("Trevor McKay <trmckay@calpoly.edu>")
    .about("Emulator for the RV32I multi-cycle Otter")
    .arg("-d, --debug         'Emulates the MCU in debug mode'")
    .arg("-b, --binary=[FILE] 'Runs the specified binary'")
    .get_matches();

    match matches.occurrences_of("debug") {
        0 => (),
        _ => {
            unsafe {
                DEBUG = true;
            }
            println!("Launching in debug mode");
        }
    }

    let bin;
    if let Some(b) = matches.value_of("binary") {
        bin = b;
    }
    else {
        println!("Warning: no binary file specified, using default mem.bin");
        bin = "mem.bin";
    }

    let mut line = String::new();
    println!("Press enter to start emulator");
    std::io::stdin().read_line(&mut line).unwrap();

    emu::emulate(bin);
}