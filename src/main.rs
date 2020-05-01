use chipper::Chip8;

fn main() {
    let mut chip8 = Chip8::new();

    chip8.load_rom_from_file("./roms/BLINKY").expect("Failed to load ROM");

    for mem in chip8.memory.iter().skip(200).take(1000) {
        println!("{:X}", mem);
    }
}
