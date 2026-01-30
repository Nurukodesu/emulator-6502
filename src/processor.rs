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
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    pc: u16,
    p: u8
}

impl Processor{

    pub fn new() -> Processor{
        Processor{
            a: 0,
            x: 0,
            y: 0,
            s: 0xFF,
            pc: 0x8000,
            p: 0
        }
    }

    fn setN(&mut self, value: u8){
        self.p = if value==N{ self.p|N} else {self.p&!N };
    }

    fn setV(&mut self, value: u8){
        self.p = if value==V{ self.p|V} else {self.p&!V };
    }

    fn setD(&mut self, value: u8){
        self.p = if value==D{ self.p|D} else {self.p&!D };
    }

    fn setI(&mut self, value: u8){
        self.p = if value==I{ self.p|I} else {self.p&!I };
    }

    fn setZ(&mut self, value: u8){
        self.p = if value==Z{ self.p|Z} else {self.p&!Z };
    }

    fn setC(&mut self, value: u8){
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
        mem.write(0x0100|self.s as u16, value);
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
        self.pc = self.pc.wrapping_add(2);
        addr.wrapping_add(self.y) as u16
    }
    
    fn abs(&mut self, mem: &Memory) -> u16{
        let addr = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr
    }
    
    fn absx(&mut self, mem: &Memory) -> u16{
        let addr = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr.wrapping_add(self.x as u16)
    }
    
    fn absy(&mut self, mem: &Memory) -> u16{
        let addr = self.read_u16(mem, self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr.wrapping_add(self.y as u16)
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
        (high << 8 | low).wrapping_add(self.y as u16)
    }
    
    fn rel(&mut self, mem: &Memory) -> u16{
        let offset = self.read_i8(mem, self.pc) as i16;
        self.pc = self.pc.wrapping_add(1);
        ((self.pc as i16)+offset) as u16
    }
    
    // Instructions
    fn lda_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lda_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a = self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }

    fn ldx_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.x = self.read(mem, addr);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn ldx_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.x = self.read(mem, addr);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn ldx_zpy(&mut self, mem: &Memory){
        let addr = self.zpy(mem);
        self.x = self.read(mem, addr);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn ldx_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.x = self.read(mem, addr);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn ldx_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.x = self.read(mem, addr);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn ldy_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.y = self.read(mem, addr);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn ldy_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.y = self.read(mem, addr);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn ldy_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.y = self.read(mem, addr);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn ldy_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.y = self.read(mem, addr);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn ldy_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.y = self.read(mem, addr);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
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
        let addr = self.absx(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_absy(&mut self, mem: &mut Memory){
        let addr = self.absy(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_indx(&mut self, mem: &mut Memory){
        let addr = self.indx(mem);
        self.write(mem, addr, self.a);
    }
    fn sta_indy(&mut self, mem: &mut Memory){
        let addr = self.indy(mem);
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
        self.push(mem, self.a | U);
    }

    fn tax(&mut self){
        self.x = self.a;
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn tay(&mut self){
        self.y = self.a;
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn txa(&mut self){
        self.a = self.x;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn txy(&mut self){
        self.a = self.y;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn tsx(&mut self){
        self.x = self.s;
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn txs(&mut self){
        self.s = self.x;
    }

    fn pla(&mut self, mem: &Memory){
        self.a = self.pull(mem);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn plp(&mut self, mem: &Memory){
        self.p = self.pull(mem);
    }
   
    fn and_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn and_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a &= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    
    fn eor_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn eor_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a ^= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }

    fn ora_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ora_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        self.a |= self.read(mem, addr);
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    
    fn bit_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setZ(if self.a&m==0{Z}else{0});
        self.setV(m&V);
        self.setN(m&N);
    }
    fn bit_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setZ(if self.a&m==0{Z}else{0});
        self.setV(m&V);
        self.setN(m&N);
    }
    
    fn adc_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn adc_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        let c = self.p&C;
        let sum = self.a as u16 + m as u16 + c as u16;
        self.setC(if sum&0x100!=0{C}else{0});
        let overflow= !(self.a^m) & (self.a^sum as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = sum as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    
    
    fn sdc_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn sdc_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        let m_inverse = m ^ 0xFF;
        let c = self.p&C;
        let diff = self.a as u16 + m_inverse as u16 + c as u16;
        self.setC(if diff&0x100!=0{C}else{0});
        let overflow= ((diff as u8)^m_inverse) & (self.a^diff as u8) & N !=0;
        self.setV(if overflow{V}else{0});
        self.a = diff as u8;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    
    fn cmp_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn cmp_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.a >= m {C} else {0});
        self.setZ(if self.a == m {C} else {0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    
    fn cpx_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn cpx_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.x >= m {C} else {0});
        self.setZ(if self.x == m {C} else {0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    
    fn cpy_imm(&mut self, mem: &Memory){
        let addr = self.imm();
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_zp(&mut self, mem: &Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_zpx(&mut self, mem: &Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_abs(&mut self, mem: &Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_absx(&mut self, mem: &Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_absy(&mut self, mem: &Memory){
        let addr = self.absy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_indx(&mut self, mem: &Memory){
        let addr = self.indx(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    fn cpy_indy(&mut self, mem: &Memory){
        let addr = self.indy(mem);
        let m = self.read(mem, addr);
        self.setC(if self.y >= m {C} else {0});
        self.setZ(if self.y == m {C} else {0});
        self.setN(if self.y&N!=0{N}else{0});
    }

    fn inc_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn inc_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn inc_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn inc_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let value = self.read(mem, addr).wrapping_add(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    
    fn inx(&mut self){
        self.x = self.x.wrapping_add(1);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn iny(&mut self){
        self.y = self.y.wrapping_add(1);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }

    fn dec_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn dec_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn dec_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    fn dec_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let value = self.read(mem, addr).wrapping_sub(1);
        self.write(mem, addr, value);
        self.setZ(if value==0{Z}else{0});
        self.setN(if value&N!=0{N}else{0});
    }
    
    fn dex(&mut self){
        self.x = self.x.wrapping_sub(1);
        self.setZ(if self.x==0{Z}else{0});
        self.setN(if self.x&N!=0{N}else{0});
    }
    fn dey(&mut self){
        self.y = self.y.wrapping_sub(1);
        self.setZ(if self.y==0{Z}else{0});
        self.setN(if self.y&N!=0{N}else{0});
    }
    
    fn asl(&mut self){
        self.setC(if self.a&0x80==1 {C} else {0});
        self.a <<= 1;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn asl_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m<<1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m<<1==0{Z}else{0});
        self.setN(if (m<<1)&N!=0{N}else{0});
    }
    fn asl_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m<<1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m<<1==0{Z}else{0});
        self.setN(if (m<<1)&N!=0{N}else{0});
    }
    fn asl_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m<<1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m<<1==0{Z}else{0});
        self.setN(if (m<<1)&N!=0{N}else{0});
    }
    fn asl_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m<<1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m<<1==0{Z}else{0});
        self.setN(if (m<<1)&N!=0{N}else{0});
    }

    fn lsr(&mut self){
        self.setC(if self.a&0x80==1 {C} else {0});
        self.a >>= 1;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn lsr_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m>>1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m>>1==0{Z}else{0});
        self.setN(if (m>>1)&N!=0{N}else{0});
    }
    fn lsr_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m>>1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m>>1==0{Z}else{0});
        self.setN(if (m>>1)&N!=0{N}else{0});
    }
    fn lsr_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m>>1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m>>1==0{Z}else{0});
        self.setN(if (m>>1)&N!=0{N}else{0});
    }
    fn lsr_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.write(mem, addr, m>>1);
        self.setC(if m&0x80==1 {C} else {0});
        self.setZ(if m>>1==0{Z}else{0});
        self.setN(if (m>>1)&N!=0{N}else{0});
    }
    
    fn rol(&mut self){
        self.setC(if self.a&0x80==1 {C} else {0});
        let c = self.p&C;
        self.a = (self.a<<1)|c;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn rol_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = self.p&C;
        let m1 = (m<<1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn rol_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = self.p&C;
        let m1 = (m<<1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn rol_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = self.p&C;
        let m1 = (m<<1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn rol_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = self.p&C;
        let m1 = (m<<1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }

    fn ror(&mut self){
        self.setC(if self.a&1==1 {C} else {0});
        let c = self.p&C;
        self.a = (self.a>>1)|c;
        self.setZ(if self.a==0{Z}else{0});
        self.setN(if self.a&N!=0{N}else{0});
    }
    fn ror_zp(&mut self, mem: &mut Memory){
        let addr = self.zp(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x1==1 {C} else {0});
        let c = (self.p&C)<<7;
        let m1 = (m>>1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn ror_zpx(&mut self, mem: &mut Memory){
        let addr = self.zpx(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = (self.p&C)<<7;
        let m1 = (m>>1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn ror_abs(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = (self.p&C)<<7;
        let m1 = (m>>1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }
    fn ror_absx(&mut self, mem: &mut Memory){
        let addr = self.absx(mem);
        let m = self.read(mem, addr);
        self.setC(if m&0x80==1 {C} else {0});
        let c = (self.p&C)<<7;
        let m1 = (m>>1)|c;
        self.write(mem, addr, m1);
        self.setZ(if m1==0{Z}else{0});
        self.setN(if m1&N!=0{N}else{0});
    }

    fn jmp_abs(&mut self, mem: &Memory){
        self.pc = self.abs(mem);
    }
    fn jmp_ind(&mut self, mem: &Memory){
        self.pc = self.ind(mem);
    }

    fn jsr(&mut self, mem: &mut Memory){
        let addr = self.abs(mem);
        let bytes = (self.pc-1).to_le_bytes();
        self.push(mem, bytes[0]);
        self.push(mem, bytes[1]);
        self.pc = addr;
    }
    
    fn rts(&mut self, mem: &Memory){
        let lo = self.pull(mem) as u16;
        let hi = self.pull(mem) as u16;
        self.pc = hi<<8 + lo;
    }
 
    fn bcs(&mut self, mem: &Memory){
        if self.p&C==C{
            self.pc  = self.pc.wrapping_add(self.rel(mem));
        }
    }
    fn bcc(&mut self, mem: &Memory){
        if self.p&C!=C{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn beq(&mut self, mem: &Memory){
        if self.p&Z==Z{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn bne(&mut self, mem: &Memory){
        if self.p&Z!=Z{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn bmi(&mut self, mem: &Memory){
        if self.p&N==N{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn bpl(&mut self, mem: &Memory){
        if self.p&N!=N{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn bvs(&mut self, mem: &Memory){
        if self.p&V==N{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
        }
    }
    fn bvc(&mut self, mem: &Memory){
        if self.p&V==V{
            self.pc  = self.pc.wrapping_add(self.rel(mem));            
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
    fn sev(&mut self){
        self.p = self.p|V;
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

    pub fn step(&mut self, mem: &mut Memory){
        let opcode = mem.read(self.pc);
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
            TYA => self.tay(),
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
            _ => {panic!("Unknown opcode {:02X}", opcode)}
        }
    }
}

impl Display for Processor{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "A: {:X}\nX: {:X}\nY: {:X}\nPC: {:X}\nS: {:X}\nP: {:b}", self.a, self.x, self.y, self.pc, self.s, self.p)
    }
}