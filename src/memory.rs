pub struct Memory{
    data: [u8; 65536]
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
}