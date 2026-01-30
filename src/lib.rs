use std::fs;

use crate::memory::Memory;

pub mod processor;
pub mod memory;
pub mod op;

pub fn read_rom(path: &str) -> Vec<u8> {
    fs::read(path).expect("Reading ROM failed")
}

pub fn load_rom(bus: &mut Memory, rom: &[u8], begin: u16){
    rom.iter().enumerate().for_each(|(i, &byte)|{
        let addr = begin.wrapping_add(i as u16);
        bus.write(addr, byte);
    });
}