use std::io::{self};

use emulator_6502::{load_nes, memory::Memory, processor::Processor, read_rom};

fn main() {
    let rom = read_rom("test/nestest.nes");
    let mut mem = Memory::new();
    load_nes(&mut mem, &rom);
    let mut cpu = Processor::nes();
    
    println!("{}\n", cpu);
    
    loop{
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        if buf.trim().eq("e"){
            break;
        }

        match buf.trim().parse::<u8>() {
            Ok(value) => {
                mem.display_pg(value);
                continue;
            },
            Err(_) => ()
        }


        cpu.step(&mut mem);
        println!("{}\n", cpu);
    }
}
