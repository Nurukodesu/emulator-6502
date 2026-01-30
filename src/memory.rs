use std::os::linux::net::SocketAddrExt;

#[derive(Debug)]
pub struct Memory{
    data: [u8; 65536]
}

fn get_pgaddr(page: u8) -> u16 {
    (page as u16)<<8
}

impl Memory{
    pub fn new() -> Memory{
        Memory{data: [0;65536]}
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn read_i8(&self, addr: u16) -> i8 {
        self.read(addr) as i8
    }
    
    pub fn read_u16(&self, addr:u16) -> u16 {
        ((self.read(addr.wrapping_add(1)) as u16)<<8)+self.read(addr) as u16
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }

    pub fn display_pg(&self, page: u8){
        let start= get_pgaddr(page) as usize;
        let end = (get_pgaddr(page) | 0xFF) as usize;
        println!("{:x?}", &self.data[start..end]);
    }
}