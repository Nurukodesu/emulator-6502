use std::fmt::Display;
use std::{ops::Add};

use crate::memory::{Memory};
use crate::op::*;

const N: u8 = 0x80;
const V: u8 = 0x40;
const U: u8 = 0x20;
const B: u8 = 0x10;
const D: u8 = 0x08;
const I: u8 = 0x04;
const Z: u8 = 0x02;
const C: u8 = 0x01;
pub struct Processor{
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub pc: u16,
    pub p: u8,
    pub cycles: u32,
    page_crossed: bool
}

impl Processor{

    pub fn new() -> Processor{
        Processor{
            a: 0,
            x: 0,
            y: 0,
            s: 0xFF,
            pc: 0x8000,
            p: 0,
            cycles: 0,
            page_crossed: false
        }
    }
    pub fn nes() -> Processor{
        Processor{
            a: 0,
            x: 0,
            y: 0,
            s: 0xFD,
            pc: 0xC000,
            p: 0x24,
            cycles: 0,
            page_crossed: false
        }
    }
    
    pub fn page_crossed(&self) -> bool{
        self.page_crossed
    }

    fn setn(&mut self, value: u8){
        self.p = if value==N{ self.p|N} else {self.p&!N };
    }

    fn setv(&mut self, value: u8){
        self.p = if value==V{ self.p|V} else {self.p&!V };
    }

    fn setz(&mut self, value: u8){
        self.p = if value==Z{ self.p|Z} else {self.p&!Z };
    }

    fn setc(&mut self, value: u8){
        self.p = if value==C{ self.p|C} else {self.p&!C };
    }

    fn read(&self, mem: &Memory, addr: u16) -> u8 {
        mem.read(addr)
    }
    
    fn read_i8(&self, mem: &Memory, addr: u16) -> i8 {
        mem.read_i8(addr)
    }

    fn read_u16(&self, mem: &Memory, addr: u16) -> u16 {
        mem.read_u16(addr)
    }

    fn write(&self, mem: &mut Memory, addr: u16, value:u8){
        mem.write(addr, value);
    }
    
    fn push(&mut self, mem: &mut Memory, value: u8){
        mem.write(0x0100+self.s as u16, value);
        self.s = self.s.wrapping_sub(1);
    }
    
    fn pull(&mut self, mem: &Memory) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read(mem, 0x0100|self.s as u16)
    }
    
    // Addressing

    fn imm(&mut self) -> u16{
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        addr
    }

    fn zp(&mut self, mem:&Memory ) -> u16{
        let addr = self.read(mem, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr as u16
    }

    fn zpx(&mut self, mem:&Memory) -> u16{
        let addr = self.read(mem, self.pc);
        self.pc = self.pc.add(1);
        addr.wrapping_add(self.x) as u16
    }

    fn zpy(&mut self, mem:&Memory) -> u16{
        let addr = self.read(mem, self.pc);
        self.pc = self.pc.wrapping_add(1);
        addr.wrapping_add(self.y) as u16
    }
    
    fn abs(&mut self, mem: &Memory) -> u16{
        let addr = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr
    }
    
    fn absx(&mut self, mem: &Memory) -> u16{
        let base = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        let addr = base.wrapping_add(self.x as u16);
        if base&0xFF00 != addr &0xFF00{
            self.cycles+=1;
        }
        addr
    }
    
    fn absx_ro(&mut self, mem: &Memory) -> u16{
        let base = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        let addr = base.wrapping_add(self.x as u16);
        addr
    }
    
    fn absy(&mut self, mem: &Memory) -> u16{
        let base= self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        let addr = base.wrapping_add(self.y as u16);
        if base&0xFF00 != addr &0xFF00{
            self.cycles+=1;
        }
        addr
    }

    fn absy_ro(&mut self, mem: &Memory) -> u16{
        let base= self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        let addr = base.wrapping_add(self.y as u16);
        addr
    }
    
    fn ind(&mut self, mem: &Memory) -> u16{
        let addr = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        // addr & 0xFF00 means keep the higher order and (addr + 1) & 0x00FF means don't add the carry bit to higher order
        (((self.read(mem, addr & 0xFF00 | ((addr+1) & 0x00FF))) as u16) << 8 ) | self.read(mem, addr) as u16
    }
    
    fn indx(&mut self, mem: &Memory) -> u16{
        let zp = self.read(mem, self.pc);
        let addr = zp.wrapping_add(self.x);
        self.pc = self.pc.wrapping_add(1);
        ((self.read(mem, addr.wrapping_add(1) as u16) as u16) << 8 ) | self.read(mem, addr as u16) as u16
    }
    
    fn indy(&mut self, mem: &Memory) -> u16{
        let zp = self.read(mem, self.pc);
        let high = self.read(mem, zp.wrapping_add(1) as u16) as u16;
        let low = self.read(mem, zp as u16) as u16;
        let base = high << 8 | low ;
        let addr = base.wrapping_add(self.y as u16);
        if base&0xFF00 != addr &0xFF00{
            self.cycles+=1;
        }
        self.pc = self.pc.wrapping_add(1);
        addr
    }

    fn indy_ro(&mut self, mem: &Memory) -> u16{
        let zp = self.read(mem, self.pc);
        let high = self.read(mem, zp.wrapping_add(1) as u16) as u16;
        let low = self.read(mem, zp as u16) as u16;
        let base = high << 8 | low ;
        let addr = base.wrapping_add(self.y as u16);
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    
    fn rel(&mut self, mem: &Memory) -> u16{
        let offset = self.read_i8(mem, self.pc) as i16;
        self.pc = self.pc.wrapping_add(1);
        ((self.pc as i16).wrapping_add(offset)) as u16
    }
    
    // Instructions
    fn lda_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lda_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a = self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn ldx_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.x = self.read(mem, addr);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn ldx_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.x = self.read(mem, addr);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn ldx_zpy(&mut self, mem: &Memory){
        let addr = self.zpy(mem);
        self.x = self.read(mem, addr);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn ldx_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.x = self.read(mem, addr);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn ldx_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.x = self.read(mem, addr);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn ldy_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.y = self.read(mem, addr);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    fn ldy_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.y = self.read(mem, addr);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    fn ldy_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.y = self.read(mem, addr);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    } fn ldy_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.y = self.read(mem, addr);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    fn ldy_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.y = self.read(mem, addr);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    
    fn sta_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_absy(&mut self, mem: &mut Memory){
        let addr = self.absy_ro(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_indy(&mut self, mem: &mut Memory){
        let addr = self.indy_ro(mem);
        self.write(mem, addr, self.a);
    }

    fn stx_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        self.write(mem, addr, self.x);
    }
    fn stx_zpy(&mut self, mem: &mut Memory){
        let addr = self.zpy(mem);
        self.write(mem, addr, self.x);
    }
    fn stx_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        self.write(mem, addr, self.x);
    }

    fn sty_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        self.write(mem, addr, self.y);
    }
    fn sty_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        self.write(mem, addr, self.y);
    }
    fn sty_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        self.write(mem, addr, self.y);
    }
    
    fn pha(&mut self, mem: &mut Memory){
        self.push(mem, self.a);
    }
    fn php(&mut self, mem: &mut Memory){
        self.push(mem, self.p | U | B);
    }

    fn tax(&mut self){
        self.x = self.a;
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn tay(&mut self){
        self.y = self.a;
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    fn txa(&mut self){
        self.a = self.x;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn tya(&mut self){
        self.a = self.y;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn tsx(&mut self){
        self.x = self.s;
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn txs(&mut self){
        self.s = self.x;
    }

    fn pla(&mut self, mem: &Memory){
        self.a = self.pull(mem);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn plp(&mut self, mem: &Memory){
        self.p = self.pull(mem)&!B|U;
    }
   
    fn and_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn and_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a &= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    
    fn eor_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn eor_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a ^= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn ora_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ora_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a |= self.read(mem, addr);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    
    fn bit_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setz(if self.a&m==0{Z}else{0});
        self.setv(m&V);
        self.setn(m&N);
    }
    fn bit_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setz(if self.a&m==0{Z}else{0});
        self.setv(m&V);
        self.setn(m&N);
    }
    
    fn adc_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn adc_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    
    
    fn sbc_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sbc_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    
    fn cmp_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cmp_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    
    fn cpx_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setc(if self.x >= m {C} else {0});
        self.setz(if self.x == m {Z} else {0});
        let diff = self.x.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cpx_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if self.x >= m {C} else {0});
        self.setz(if self.x == m {Z} else {0});
        let diff = self.x.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cpx_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if self.x >= m {C} else {0});
        self.setz(if self.x == m {Z} else {0});
        let diff = self.x.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    
    fn cpy_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setc(if self.y >= m {C} else {0});
        self.setz(if self.y == m {Z} else {0});
        let diff = self.y.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cpy_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if self.y >= m {C} else {0});
        self.setz(if self.y == m {Z} else {0});
        let diff = self.y.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn cpy_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if self.y >= m {C} else {0});
        self.setz(if self.y == m {Z} else {0});
        let diff = self.y.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }

    fn inc_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn inc_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn inc_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn inc_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    
    fn inx(&mut self){
        self.x = self.x.wrapping_add(1);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn iny(&mut self){
        self.y = self.y.wrapping_add(1);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }

    fn dec_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn dec_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn dec_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    fn dec_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setz(if value==0{Z}else{0});
        self.setn(if value&N!=0{N}else{0});
    }
    
    fn dex(&mut self){
        self.x = self.x.wrapping_sub(1);
        self.setz(if self.x==0{Z}else{0});
        self.setn(if self.x&N!=0{N}else{0});
    }
    fn dey(&mut self){
        self.y = self.y.wrapping_sub(1);
        self.setz(if self.y==0{Z}else{0});
        self.setn(if self.y&N!=0{N}else{0});
    }
    
    fn asl(&mut self){
        self.setc(if self.a&0x80==0x80 {C} else {0});
        self.a = self.a.wrapping_shl(1);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn asl_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }
    fn asl_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }
    fn asl_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }
    fn asl_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }

    fn lsr(&mut self){
        self.setc(if self.a&C==C {C} else {0});
        self.a = self.a.wrapping_shr(1);
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lsr_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }
    fn lsr_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if (m)&N!=0{N}else{0});
    }
    fn lsr_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if (m)&N!=0{N}else{0});
    }
    fn lsr_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.setz(if m==0{Z}else{0});
        self.setn(if m&N!=0{N}else{0});
    }
    
    fn rol(&mut self){
        let c = self.p&C;
        self.setc(if self.a&0x80==0x80 {C} else {0});
        self.a = self.a.wrapping_shl(1)|c;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rol_zp(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn rol_zpx(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn rol_abs(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn rol_absx(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }

    fn ror(&mut self){
        let c = (self.p&C)<<7;
        self.setc(if self.a&1==1 {C} else {0});
        self.a = self.a.wrapping_shr(1)|c;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn ror_zp(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7;
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m1 = self.a.wrapping_shr(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn ror_zpx(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7;
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m1 = self.a.wrapping_shr(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn ror_abs(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7;
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m1 = self.a.wrapping_shr(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }
    fn ror_absx(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7;
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m1 = self.a.wrapping_shr(1)|c;
        self.write(mem, addr, m1);
        self.setz(if m1==0{Z}else{0});
        self.setn(if m1&N!=0{N}else{0});
    }

    fn jmp_abs(&mut self, mem: &Memory){
        self.pc = self.abs(mem);
    }
    fn jmp_ind(&mut self, mem: &Memory){
        self.pc = self.ind(mem);
    }

    fn jsr(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let bytes = (self.pc-1).to_be_bytes();
        self.push(mem, bytes[0]);
        self.push(mem, bytes[1]);
        self.pc = addr;
    }
    
    fn rts(&mut self, mem: &Memory){
        let lo = self.pull(mem) as u16;
        let hi = self.pull(mem) as u16;
        self.pc = hi <<8 | lo;
        self.pc = self.pc.wrapping_add(1);
    }
 
    fn bcs(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&C==C{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bcc(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&C!=C{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn beq(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&Z==Z{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bne(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&Z!=Z{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bmi(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&N==N{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bpl(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&N!=N{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bvs(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&V==V{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    fn bvc(&mut self, mem: &Memory){
        let old_pc = self.pc;
        let pc = self.rel(mem);
        if self.p&V!=V{
            self.pc  = pc;
            self.cycles += 1;
            if (old_pc & 0xFF00) != (pc & 0xFF00) {
                self.cycles += 1;
                self.page_crossed = true;
            }
        }
    }
    
    fn clc(&mut self){
        self.p = self.p&!C;
    }
    fn cld(&mut self){
        self.p = self.p&!D;
    }
    fn cli(&mut self){
        self.p = self.p&!I;
    }
    fn clv(&mut self){
        self.p = self.p&!V;
    }
    
    fn sec(&mut self){
        self.p = self.p|C;
    }
    fn sed(&mut self){
        self.p = self.p|D;
    }
    fn sei(&mut self){
        self.p = self.p|I;
    }
    
    fn brk(&mut self, mem: &mut Memory){
        self.pc = self.pc.wrapping_add(1);
        let bytes = self.pc.to_le_bytes();
        self.push(mem, bytes[0]);
        self.push(mem, bytes[1]);
        self.push(mem, self.p|B|U);
        self.p |= I;
    }
    
    fn rti(&mut self, mem: &Memory) {
        self.p = (self.pull(mem) & !B) | U;
        let lo = self.pull(mem) as u16;
        let hi = self.pull(mem) as u16;
        self.pc = (hi << 8) | lo
    }
    
    fn lax_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lax_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lax_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lax_zpy(&mut self, mem: &Memory){
        let addr = self.zpy(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lax_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn lax_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a = self.read(mem, addr);
        self.x = self.a;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn sax_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        self.write(mem, addr, self.a&self.x);
    }
    fn sax_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        self.write(mem, addr, self.a&self.x);
    }
    fn sax_zpy(&mut self, mem: &mut Memory){
        let addr = self.zpy(mem);
        self.write(mem, addr, self.a&self.x);
    }
    fn sax_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        self.write(mem, addr, self.a&self.x);
    }

    fn dcp_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_absy(&mut self, mem: &mut Memory){
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }
    fn dcp_indy(&mut self, mem: &mut Memory){
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, m);
        self.setc(if self.a >= m {C} else {0});
        self.setz(if self.a == m {Z} else {0});
        let diff = self.a.wrapping_sub(m);
        self.setn(if diff&N!=0{N}else{0});
    }

    fn isc_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_absy(&mut self, mem: &mut Memory){
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn isc_indy(&mut self, mem: &mut Memory){
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, m);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setc(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = diff as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn slo_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_absy(&mut self, mem: &mut Memory){
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn slo_indy(&mut self, mem: &mut Memory){
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m = m.wrapping_shl(1);
        self.write(mem, addr, m);
        self.a |= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn rla_zp(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_zpx(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_abs(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_absx(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_absy(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_indx(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rla_indy(&mut self, mem: &mut Memory){
        let c = self.p&C;
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x80==0x80 {C} else {0});
        let m1 = m.wrapping_shl(1)|c;
        self.write(mem, addr, m1);
        self.a &= m1;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    
    fn sre_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_absx(&mut self, mem: &mut Memory){
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_absy(&mut self, mem: &mut Memory){
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn sre_indy(&mut self, mem: &mut Memory){
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&C==C {C} else {0});
        let m = m.wrapping_shr(1);
        self.write(mem, addr, m);
        self.a ^= m;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }

    fn rra_zp(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_zpx(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_abs(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_absx(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.absx_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_absy(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.absy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_indx(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }
    fn rra_indy(&mut self, mem: &mut Memory){
        let c = (self.p&C)<<7; // old c
        let addr = self.indy_ro(mem);
        let m = self.read(mem, addr);
        self.setc(if m&0x1==1 {C} else {0});
        let m = m.wrapping_shr(1)|c;
        self.write(mem, addr, m);
        let c = self.p&C; // new c set by ror
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setc(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setv(if overflow{V}else{0});
        self.a = sum as u8;
        self.setz(if self.a==0{Z}else{0});
        self.setn(if self.a&N!=0{N}else{0});
    }    

    pub fn step(&mut self, mem: &mut Memory){
        let opcode = mem.read(self.pc);
        self.page_crossed = false;
        self.cycles = BASE_CYCLES[opcode as usize] as u32;
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            LDA_IMM => self.lda_imm(mem),
            LDA_ZP => self.lda_zp(mem),
            LDA_ZPX => self.lda_zpx(mem),
            LDA_ABS => self.lda_abs(mem),
            LDA_ABSX => self.lda_absx(mem),
            LDA_ABSY => self.lda_absy(mem),
            LDA_INDX => self.lda_indx(mem),
            LDA_INDY => self.lda_indy(mem),
            
            LDX_IMM => self.ldx_imm(mem),
            LDX_ZP => self.ldx_zp(mem),
            LDX_ZPY => self.ldx_zpy(mem),
            LDX_ABS => self.ldx_abs(mem),
            LDX_ABSY => self.ldx_absy(mem),
            
            LDY_IMM => self.ldy_imm(mem),
            LDY_ZP => self.ldy_zp(mem),
            LDY_ZPX => self.ldy_zpx(mem),
            LDY_ABS => self.ldy_abs(mem),
            LDY_ABSX => self.ldy_absx(mem),
            
            STA_ZP => self.sta_zp(mem),
            STA_ZPX => self.sta_zpx(mem),
            STA_ABS => self.sta_abs(mem),
            STA_ABSX => self.sta_absx(mem),
            STA_ABSY => self.sta_absy(mem),
            STA_INDX => self.sta_indx(mem),
            STA_INDY => self.sta_indy(mem),
            
            STX_ZP => self.stx_zp(mem),
            STX_ZPY => self.stx_zpy(mem),
            STX_ABS => self.stx_abs(mem),

            STY_ZP => self.sty_zp(mem),
            STY_ZPX => self.sty_zpx(mem),
            STY_ABS => self.sty_abs(mem),
            
            TAX => self.tax(),
            TXA => self.txa(),
            TAY => self.tay(),
            TYA => self.tya(),
            TSX => self.tsx(),
            TXS => self.txs(),
            
            PHA => self.pha(mem),
            PHP => self.php(mem),
            PLA => self.pla(mem),
            PLP => self.plp(mem),

            ORA_IMM => self.ora_imm(mem),
            ORA_ZP => self.ora_zp(mem),
            ORA_ZPX => self.ora_zpx(mem),
            ORA_ABS => self.ora_abs(mem),
            ORA_ABSX => self.ora_absx(mem),
            ORA_ABSY => self.ora_absy(mem),
            ORA_INDX => self.ora_indx(mem),
            ORA_INDY => self.ora_indy(mem),

            AND_IMM => self.and_imm(mem),
            AND_ZP => self.and_zp(mem),
            AND_ZPX => self.and_zpx(mem),
            AND_ABS => self.and_abs(mem),
            AND_ABSX => self.and_absx(mem),
            AND_ABSY => self.and_absy(mem),
            AND_INDX => self.and_indx(mem),
            AND_INDY => self.and_indy(mem),

            EOR_IMM => self.eor_imm(mem),
            EOR_ZP => self.eor_zp(mem),
            EOR_ZPX => self.eor_zpx(mem),
            EOR_ABS => self.eor_abs(mem),
            EOR_ABSX => self.eor_absx(mem),
            EOR_ABSY => self.eor_absy(mem),
            EOR_INDX => self.eor_indx(mem),
            EOR_INDY => self.eor_indy(mem),
            
            BIT_ZP => self.bit_zp(mem),
            BIT_ABS => self.bit_abs(mem),
            
            ADC_IMM => self.adc_imm(mem),
            ADC_ZP => self.adc_zp(mem),
            ADC_ZPX => self.adc_zpx(mem),
            ADC_ABS => self.adc_abs(mem),
            ADC_ABSX => self.adc_absx(mem),
            ADC_ABSY => self.adc_absy(mem),
            ADC_INDX => self.adc_indx(mem),
            ADC_INDY => self.adc_indy(mem),

            SBC_IMM | 0xEB => self.sbc_imm(mem),
            SBC_ZP => self.sbc_zp(mem),
            SBC_ZPX => self.sbc_zpx(mem),
            SBC_ABS => self.sbc_abs(mem),
            SBC_ABSX => self.sbc_absx(mem),
            SBC_ABSY => self.sbc_absy(mem),
            SBC_INDX => self.sbc_indx(mem),
            SBC_INDY => self.sbc_indy(mem),

            CMP_IMM => self.cmp_imm(mem),
            CMP_ZP => self.cmp_zp(mem),
            CMP_ZPX => self.cmp_zpx(mem),
            CMP_ABS => self.cmp_abs(mem),
            CMP_ABSX => self.cmp_absx(mem),
            CMP_ABSY => self.cmp_absy(mem),
            CMP_INDX => self.cmp_indx(mem),
            CMP_INDY => self.cmp_indy(mem),
            
            CPX_IMM => self.cpx_imm(mem),
            CPX_ZP => self.cpx_zp(mem),
            CPX_ABS => self.cpx_abs(mem),

            CPY_IMM => self.cpy_imm(mem),
            CPY_ZP => self.cpy_zp(mem),
            CPY_ABS => self.cpy_abs(mem),
            
            INC_ZP => self.inc_zp(mem),
            INC_ZPX => self.inc_zpx(mem),
            INC_ABS => self.inc_abs(mem),
            INC_ABSX => self.inc_absx(mem),
            INX => self.inx(),
            INY => self.iny(),

            DEC_ZP => self.dec_zp(mem),
            DEC_ZPX => self.dec_zpx(mem),
            DEC_ABS => self.dec_abs(mem),
            DEC_ABSX => self.dec_absx(mem),
            DEX => self.dex(),
            DEY => self.dey(),
            
            ASL => self.asl(),
            ASL_ZP => self.asl_zp(mem),
            ASL_ZPX => self.asl_zpx(mem),
            ASL_ABS => self.asl_abs(mem),
            ASL_ABSX => self.asl_absx(mem),

            LSR => self.lsr(),
            LSR_ZP => self.lsr_zp(mem),
            LSR_ZPX => self.lsr_zpx(mem),
            LSR_ABS => self.lsr_abs(mem),
            LSR_ABSX => self.lsr_absx(mem),

            ROL => self.rol(),
            ROL_ZP => self.rol_zp(mem),
            ROL_ZPX => self.rol_zpx(mem),
            ROL_ABS => self.rol_abs(mem),
            ROL_ABSX => self.rol_absx(mem),

            ROR => self.ror(),
            ROR_ZP => self.ror_zp(mem),
            ROR_ZPX => self.ror_zpx(mem),
            ROR_ABS => self.ror_abs(mem),
            ROR_ABSX => self.ror_absx(mem),
            
            JSR => self.jsr(mem),
            RTS => self.rts(mem),
            JMP_ABS => self.jmp_abs(mem),
            JMP_IND => self.jmp_ind(mem),
            
            BPL => self.bpl(mem),
            BMI => self.bmi(mem),
            BEQ => self.beq(mem),
            BNE => self.bne(mem),
            BCC => self.bcc(mem),
            BCS => self.bcs(mem),
            BVC => self.bvc(mem),
            BVS => self.bvs(mem),
            
            CLC => self.clc(),
            CLD => self.cld(),
            CLI => self.cli(),
            CLV => self.clv(),
            
            SEC => self.sec(),
            SED => self.sed(),
            SEI => self.sei(),
            
            BRK => self.brk(mem),
            RTI => self.rti(mem),
            NOP => (),
            
            // Illegal opcodes:
            0x04 | 0x44 | 0x64 => {self.zp(mem);}, // NOP ZP
            0x0C => {self.abs(mem);}, //NOP ABS
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {self.zpx(mem);}, //NOP ZPX
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {self.absx(mem);}, //NOP ABSX
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => (), // NOP ACC
            0x80 => {self.imm();}, //NOP IMM
            
            LAX_ZP => self.lax_zp(mem),
            LAX_ZPY => self.lax_zpy(mem),
            LAX_ABS => self.lax_abs(mem),
            LAX_ABSY => self.lax_absy(mem),
            LAX_INDX => self.lax_indx(mem),
            LAX_INDY => self.lax_indy(mem),

            SAX_ZP => self.sax_zp(mem),
            SAX_ZPY => self.sax_zpy(mem),
            SAX_ABS => self.sax_abs(mem),
            SAX_INDX => self.sax_indx(mem),
            
            DCP_ZP => self.dcp_zp(mem),
            DCP_ZPX => self.dcp_zpx(mem),
            DCP_ABS => self.dcp_abs(mem),
            DCP_ABSX => self.dcp_absx(mem),
            DCP_ABSY => self.dcp_absy(mem),
            DCP_INDX => self.dcp_indx(mem),
            DCP_INDY => self.dcp_indy(mem),
            
            ISC_ZP => self.isc_zp(mem),
            ISC_ZPX => self.isc_zpx(mem),
            ISC_ABS => self.isc_abs(mem),
            ISC_ABSX => self.isc_absx(mem),
            ISC_ABSY => self.isc_absy(mem),
            ISC_INDX => self.isc_indx(mem),
            ISC_INDY => self.isc_indy(mem),
            
            SLO_ZP => self.slo_zp(mem),
            SLO_ZPX => self.slo_zpx(mem),
            SLO_ABS => self.slo_abs(mem),
            SLO_ABSX => self.slo_absx(mem),
            SLO_ABSY => self.slo_absy(mem),
            SLO_INDX => self.slo_indx(mem),
            SLO_INDY => self.slo_indy(mem),
            
            RLA_ZP => self.rla_zp(mem),
            RLA_ZPX => self.rla_zpx(mem),
            RLA_ABS => self.rla_abs(mem),
            RLA_ABSX => self.rla_absx(mem),
            RLA_ABSY => self.rla_absy(mem),
            RLA_INDX => self.rla_indx(mem),
            RLA_INDY => self.rla_indy(mem),

            SRE_ZP => self.sre_zp(mem),
            SRE_ZPX => self.sre_zpx(mem),
            SRE_ABS => self.sre_abs(mem),
            SRE_ABSX => self.sre_absx(mem),
            SRE_ABSY => self.sre_absy(mem),
            SRE_INDX => self.sre_indx(mem),
            SRE_INDY => self.sre_indy(mem),
            
            RRA_ZP => self.rra_zp(mem),
            RRA_ZPX => self.rra_zpx(mem),
            RRA_ABS => self.rra_abs(mem),
            RRA_ABSX => self.rra_absx(mem),
            RRA_ABSY => self.rra_absy(mem),
            RRA_INDX => self.rra_indx(mem),
            RRA_INDY => self.rra_indy(mem),
            _ => {panic!("Unknown opcode {:02X}", opcode)}
        }
    }
}

impl Display for Processor{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}        A: {:X} X: {:X} Y: {:X} P: {:X} SP: {:X}", self.pc, self.a, self.x, self.y, self.p, self.s)
    }
}