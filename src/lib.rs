use core::panic;
use std::fs;

use crate::memory::Memory;

pub mod processor;
pub mod memory;
pub mod op;

pub fn read_rom(path: &str) -> Vec<u8> {
    fs::read(path).expect("Reading ROM failed")
}

pub fn load_rom(bus: &mut Memory, rom: &[u8]){
    rom.iter().enumerate().for_each(|(i, &byte)|{
        let addr = 0x8000_u16.wrapping_add(i as u16);
        bus.write(addr, byte);
    });
}

pub fn load_rom_16kb(bus: &mut Memory, rom: &[u8]){
    let buf: Vec<u8> = rom.to_vec().iter().chain(rom.to_vec().iter().rev()).copied().collect();
    buf.iter().enumerate().for_each(|(i, &byte)|{
        let addr = 0x8000_u16.wrapping_add(i as u16);
        bus.write(addr, byte);
    });
}

pub fn load_nes(bus: &mut Memory, rom: &[u8]){
    let header = &rom[0..16];
    let prg_banks = header[4] as usize;
    let prg_size = prg_banks * 16 * 1024;
    let prg_start = 16;
    let prg_end = prg_start + prg_size;
    let prg = &rom[prg_start..prg_end];
    match prg.len() {
        0x4000 => {
            (0x8000u16..=0xFFFFu16).zip(prg.iter().cycle()).for_each(|(addr, &byte)|{bus.write(addr, byte);});
        },
        0x8000 => {
            (0x8000u16..=0xFFFFu16).zip(prg.iter()).for_each(|(addr, &byte)|{bus.write(addr, byte);});
        },
        _ => panic!("Unsupported PRG size")
    }
}