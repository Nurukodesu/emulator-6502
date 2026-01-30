use emulator_6502::{load_rom, memory::Memory, processor::Processor, read_rom};

fn main() {
    let rom = read_rom("test/ldaimmtest");
    let mut mem = Memory::new();
    load_rom(&mut mem, &rom, 0x8000);
    let mut cpu = Processor::new();
    cpu.step(&mut mem);
    println!("{}", cpu);
}
