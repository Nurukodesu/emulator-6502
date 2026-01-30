pub const LDA_IMM: u8 = 0xA9;
pub const LDA_ZP: u8 = 0xA5;
pub const LDA_ZPX: u8 = 0xB5;
pub const LDA_ABS: u8 = 0xAD;
pub const LDA_ABSX: u8 = 0xBD;
pub const LDA_ABSY: u8 = 0xB9;
pub const LDA_INDX: u8 = 0xA1;
pub const LDA_INDY: u8 = 0xB1;

pub const LDX_IMM: u8 = 0xA2;
pub const LDX_ZP: u8 = 0xA6;
pub const LDX_ZPY: u8 = 0xB6;
pub const LDX_ABS: u8 = 0xAE;
pub const LDX_ABSY: u8 = 0xBE;

pub const LDY_IMM: u8 = 0xA0;
pub const LDY_ZP: u8 = 0xA4;
pub const LDY_ZPX: u8 = 0xB4;
pub const LDY_ABS: u8 = 0xAC;
pub const LDY_ABSX: u8 = 0xBC;

pub const STA_ZP: u8 = 0x85;
pub const STA_ZPX: u8 = 0x95;
pub const STA_ABS: u8 = 0x8D;
pub const STA_ABSX: u8 = 0x9D;
pub const STA_ABSY: u8 = 0x99;
pub const STA_INDX: u8 = 0x81;
pub const STA_INDY: u8 = 0x91;

pub const STX_ZP: u8 = 0x86;
pub const STX_ZPY: u8 = 0x96;
pub const STX_ABS: u8 = 0x8E;

pub const STY_ZP: u8 = 0x84;
pub const STY_ZPX: u8 = 0x94;
pub const STY_ABS: u8 = 0x8C;

pub const TAX: u8 = 0xAA;
pub const TXA: u8 = 0x8A;
pub const TAY: u8 = 0xA8;
pub const TYA: u8 = 0x98;
pub const TSX: u8 = 0xBA;
pub const TXS: u8 = 0x9A;

pub const PHA: u8 = 0x48;
pub const PHP: u8 = 0x08;
pub const PLA: u8 = 0x68;
pub const PLP: u8 = 0x28;

pub const ORA_IMM: u8 = 0x09;
pub const ORA_ZP: u8 = 0x05;
pub const ORA_ZPX: u8 = 0x15;
pub const ORA_ABS: u8 = 0x0D;
pub const ORA_ABSX: u8 = 0x1D;
pub const ORA_ABSY: u8 = 0x19;
pub const ORA_INDX: u8 = 0x01;
pub const ORA_INDY: u8 = 0x11;

pub const AND_IMM: u8 = 0x29;
pub const AND_ZP: u8 = 0x25;
pub const AND_ZPX: u8 = 0x35;
pub const AND_ABS: u8 = 0x2D;
pub const AND_ABSX: u8 = 0x3D;
pub const AND_ABSY: u8 = 0x39;
pub const AND_INDX: u8 = 0x21;
pub const AND_INDY: u8 = 0x31;

pub const EOR_IMM: u8 = 0x49;
pub const EOR_ZP: u8 = 0x45;
pub const EOR_ZPX: u8 = 0x55;
pub const EOR_ABS: u8 = 0x4D;
pub const EOR_ABSX: u8 = 0x5D;
pub const EOR_ABSY: u8 = 0x59;
pub const EOR_INDX: u8 = 0x41;
pub const EOR_INDY: u8 = 0x51;
