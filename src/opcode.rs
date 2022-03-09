pub const LI:    u32 = 0b10_000_001;
pub const LUI:   u32 = 0b10_000_010;
pub const SYSFN: u32 = 0b10_000_011;

pub const STU8:  u32 = 0b10_000_100;
pub const STU16: u32 = 0b10_000_101;
pub const ST:    u32 = 0b10_000_110;

pub const LDS8:  u32 = 0b10_011_000;
pub const LDU8:  u32 = 0b10_011_100;
pub const LDS16: u32 = 0b10_011_001;
pub const LDU16: u32 = 0b10_011_101;
pub const LD:    u32 = 0b10_011_010;

pub const JAL:   u32 = 0b10_100_000;
pub const JALR:  u32 = 0b10_100_001;
pub const BEQ:   u32 = 0b10_100_010;
pub const BNE:   u32 = 0b10_100_011;
pub const BLT:   u32 = 0b10_100_100;
pub const BGE:   u32 = 0b10_100_101;
pub const BLTU:  u32 = 0b10_100_110;
pub const BGEU:  u32 = 0b10_100_111;

pub const ADDI:  u32 = 0b10_001_000;
pub const RSUBI: u32 = 0b10_001_001;
pub const MULI:  u32 = 0b10_001_010;

pub const ANDI:  u32 = 0b10_010_000;
pub const ORI:   u32 = 0b10_010_001;
pub const XORI:  u32 = 0b10_010_010;
pub const SHLI:  u32 = 0b10_010_011;
pub const LSHRI: u32 = 0b10_010_100;
pub const ASHRI: u32 = 0b10_010_101;

pub const ADD:   u32 = 0b10_101_000;
pub const SUB:   u32 = 0b10_101_001;
pub const MUL:   u32 = 0b10_101_010;

pub const AND:   u32 = 0b10_110_000;
pub const OR:    u32 = 0b10_110_001;
pub const XOR:   u32 = 0b10_110_010;
pub const SHL:   u32 = 0b10_110_011;
pub const LSHR:  u32 = 0b10_110_100;
pub const ASHR:  u32 = 0b10_110_101;

pub const MULW:  u32 = 0b10_111_000;
pub const MULWU: u32 = 0b10_111_001;
pub const DIV:   u32 = 0b10_111_010;
pub const DIVU:  u32 = 0b10_111_011;
