use std::{fs::File, io::{self, BufRead, BufReader}};
use emulator_6502::{load_nes, memory::Memory, processor::Processor, read_rom};
use crate::ppu::*;

macro_rules! assert_hex_eq8 {
    ($line:expr, $left:expr, $right:expr, $name:expr) => {
        if $left != $right {
            panic!(
                "Line {} | {} mismatch: expected ${:02X}, got ${:02X}",
                $line,
                $name,
                $right,
                $left
            );
        }
    };
}
macro_rules! assert_hex_eq16 {
    ($line:expr, $left:expr, $right:expr, $name:expr) => {
        if $left != $right {
            panic!(
                "Line {} | {} mismatch: expected ${:04X}, got ${:04X}",
                $line,
                $name,
                $right,
                $left
            );
        }
    };
}

pub fn nestest() -> io::Result<()>{
    let mut ppu = ClockPPU::new();

    let rom = read_rom("test/nestest.nes");
    let mut mem = Memory::new();
    load_nes(&mut mem, &rom);
    let mut cpu = Processor::nes();

    let file = File::open("test/nestest.log")?;
    let reader = BufReader::new(file);

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let line_no = line_no + 1;


        if let Some(expected) = parse_nestest_line(&line) {

            assert_hex_eq16!(line_no, cpu.pc, expected.pc, "PC");
            assert_hex_eq8!(line_no, cpu.a,  expected.a,  "A");
            assert_hex_eq8!(line_no, cpu.x,  expected.x,  "X");
            assert_hex_eq8!(line_no, cpu.y,  expected.y,  "Y");
            assert_hex_eq8!(line_no, cpu.s,  expected.sp, "SP");
            assert_hex_eq8!(line_no, cpu.p,  expected.p,  "P");

            assert_eq!(
            ppu.cyc(),
            expected.cyc,
            "Line {}: CYC mismatch (ppu={})",
            line_no,
            ppu.cyc()
            );

            cpu.step(&mut mem);
            println!("{cpu}");
            ppu.step_cpu(cpu.cycles);
        }
    }

    Ok(())
}
#[derive(Debug)]
pub struct NestestLine {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub sp: u8,
    pub cyc: u32,
}

pub fn parse_nestest_line(line: &str) -> Option<NestestLine> {
    let line = line.trim_start();

    // PC must be first 4 hex chars
    let pc = u16::from_str_radix(&line[0..4], 16).ok()?;

    let mut a = None;
    let mut x = None;
    let mut y = None;
    let mut p = None;
    let mut sp = None;
    let mut cyc = None;

    for token in line.split_whitespace() {
        if let Some(v) = token.strip_prefix("A:") {
            a = u8::from_str_radix(v, 16).ok();
        } else if let Some(v) = token.strip_prefix("X:") {
            x = u8::from_str_radix(v, 16).ok();
        } else if let Some(v) = token.strip_prefix("Y:") {
            y = u8::from_str_radix(v, 16).ok();
        } else if let Some(v) = token.strip_prefix("P:") {
            p = u8::from_str_radix(v, 16).ok();
        } else if let Some(v) = token.strip_prefix("SP:") {
            sp = u8::from_str_radix(v, 16).ok();
        } else if let Some(idx) = line.find("CYC:") {
            cyc = line[idx + 4..].trim().split_whitespace().next()?.parse().ok();
        }
    }

    Some(NestestLine {
        pc,
        a: a?,
        x: x?,
        y: y?,
        p: p?,
        sp: sp?,
        cyc: cyc?,
    })
}



