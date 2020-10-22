mod emu;

fn main() {
    println!("\nOTTER-EMU");
    println!("Version 0.1.0\n");

    emu::Otter::init("res/test_all.bin").run();
}