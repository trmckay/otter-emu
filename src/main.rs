mod emu;

fn main() {
    let bin = "res/programs/leds/leds.bin";
    emu::emulate(bin);
}