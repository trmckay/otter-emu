#[path = "./otter.rs"] mod otter;
#[path = "./cli.rs"] mod cli;
use std::{thread, time};

const RF_NAMES: [&str; 32] = [
    "zero", "ra  ", "sp  ", "gp  ", "tp  ", "t0  ", "t1  ", "t2  ",
    "s0  ", "s1  ", "a0  ", "a1  ", "a2  ", "a3  ", "a4  ", "a5  ",
    "a6  ", "a7  ", "s2  ", "s3  ", "s4  ", "s5  ", "s6  ", "s7  ",
    "s8  ", "s9  ", "s10 ", "s11 ", "t3  ", "t4  ", "t5  ", "t6  "
];

#[derive(Debug)]
pub struct Options {
    pub bin: String,
    pub debug: bool,
    pub bps: Vec<u32>,
    pub log_path: String,
    pub log_to_f: bool
}

pub fn emulate(opts: &mut Options) {
    let mut mcu = otter::MCU::new();
    mcu.load_bin(&opts.bin);
    cli::run_cli(&mut mcu, opts);
}
