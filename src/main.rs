mod mcu;

fn main() {
    println!("\nOTTER-EMU");
    println!("Version 0.1.0\n");

    let mut mcu = mcu::Otter::new();
    mcu.load_bin("res/test_all.bin");
    mcu.run();
}