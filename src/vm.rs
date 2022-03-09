use std::error;
use std::fmt;
use std::mem::{size_of, size_of_val};

use crate::opcode;

pub trait Sysfn {
    fn read(&mut self) -> u32;
    fn write(&mut self, value: u32);
}

#[derive(Clone, Debug)]
pub enum Error {
    UnknownSysfn(u32),
    UnknownInst(u32),
    InvalidAddr(u32),
    InvalidPc(u32),
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub pc: u32,
    pub regs: [u32; 16],
}

impl State {
    pub fn new() -> State {
        State { pc: 0x1000, regs: [0; 16] }
    }

    pub fn with_pc(pc: u32) -> State {
        State { pc, regs: [0; 16] }
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

pub fn run(state: &mut State, memory: &mut [u8], sysfn: &mut dyn Sysfn) -> Result<u32, Error> {
    let mut pc: u32 = state.pc;
    let regs = &mut state.regs;
    let memory_len = memory.len();
    let memory_ptr = memory.as_mut_ptr();

    macro_rules! vm_exit {
        ($result:expr) => {{
            state.pc = pc;
            return $result;
        }};
    }

    macro_rules! load_int {
        ($int:ty, $addr:expr) => {{
            let addr = $addr as usize;
            if addr.checked_add(size_of::<$int>() - 1).map_or(false, |upper| upper < memory_len) {
                Some(<$int>::from_le_bytes(unsafe { *(memory_ptr.add(addr) as *const _) }))
            } else {
                None
            }
        }};
    }

    macro_rules! store_int {
        ($value:expr, $addr:expr) => {{
            let (value, addr) = ($value, $addr as usize);
            if addr.checked_add(size_of_val(&value) - 1).map_or(false, |upper| upper < memory_len) {
                unsafe { *(memory_ptr.add(addr) as *mut _) = value.to_le_bytes() };
                Some(value)
            } else {
                None
            }
        }};
    }

    if pc % 4 != 0 {
        vm_exit!(Err(Error::InvalidPc(pc)));
    }

    loop {
        regs[0] = 0;
        let inst = match load_int!(u32, pc) {
            Some(inst) => inst,
            None => vm_exit!(Err(Error::InvalidPc(pc))),
        };
        pc = u32::wrapping_add(pc, 4);

        macro_rules! load_impl {
            ($int:ty) => {{
                let (rd, rb, off) = decode_rrc(inst);
                let addr = u32::wrapping_add(regs[rb], off);
                if let Some(value) = load_int!($int, addr) {
                    regs[rd] = value as u32;
                } else {
                    vm_exit!(Err(Error::InvalidAddr(addr)));
                }
            }};
        }

        macro_rules! store_impl {
            ($int:ty) => {{
                let (rs, rb, off) = decode_rrc(inst);
                let addr = u32::wrapping_add(regs[rb], off);
                if store_int!(regs[rs] as $int, addr).is_none() {
                    vm_exit!(Err(Error::InvalidAddr(addr)));
                }
            }};
        }

        macro_rules! branch_impl {
            ($cond:expr) => {{
                let (rs1, rs2, off) = decode_rrc(inst);
                if $cond(regs[rs1], regs[rs2]) {
                    pc = u32::wrapping_add(pc, off << 2);
                }
            }};
        }

        macro_rules! binop_imm_impl {
            ($op:expr) => {{
                let (rd, rs, imm) = decode_rrc(inst);
                regs[rd] = $op(regs[rs], imm) as u32;
            }};
        }

        macro_rules! binop_impl {
            ($op:expr) => {{
                let (rd, rs1, rs2) = decode_rrr(inst);
                regs[rd] = $op(regs[rs1], regs[rs2]) as u32;
            }};
        }

        match inst & 0xFF {
            opcode::STU8  => store_impl!(u8),
            opcode::STU16 => store_impl!(u16),
            opcode::ST    => store_impl!(u32),

            opcode::LDS8  => load_impl!(i8),
            opcode::LDU8  => load_impl!(u8),
            opcode::LDS16 => load_impl!(i16),
            opcode::LDU16 => load_impl!(u16),
            opcode::LD    => load_impl!(i32),

            opcode::BEQ   => branch_impl!(|x, y| x == y),
            opcode::BNE   => branch_impl!(|x, y| x != y),
            opcode::BLT   => branch_impl!(|x, y| (x as i32) <  (y as i32)),
            opcode::BGE   => branch_impl!(|x, y| (x as i32) >= (y as i32)),
            opcode::BLTU  => branch_impl!(|x, y| x <  y),
            opcode::BGEU  => branch_impl!(|x, y| x >= y),

            opcode::ADDI  => binop_imm_impl!(u32::wrapping_add),
            opcode::RSUBI => binop_imm_impl!(|x, y| u32::wrapping_sub(y, x)),
            opcode::MULI  => binop_imm_impl!(u32::wrapping_mul),
            opcode::ANDI  => binop_imm_impl!(|x, y| x & y),
            opcode::ORI   => binop_imm_impl!(|x, y| x | y),
            opcode::XORI  => binop_imm_impl!(|x, y| x ^ y),
            opcode::SHLI  => binop_imm_impl!(|x, y| x << (y & 0x1F)),
            opcode::LSHRI => binop_imm_impl!(|x, y| x >> (y & 0x1F)),
            opcode::ASHRI => binop_imm_impl!(|x, y| (x as i32) >> (y & 0x1F)),

            opcode::ADD   => binop_impl!(u32::wrapping_add),
            opcode::SUB   => binop_impl!(u32::wrapping_sub),
            opcode::MUL   => binop_impl!(u32::wrapping_mul),
            opcode::AND   => binop_impl!(|x, y| x & y),
            opcode::OR    => binop_impl!(|x, y| x | y),
            opcode::XOR   => binop_impl!(|x, y| x ^ y),
            opcode::SHL   => binop_impl!(|x, y| x << (y & 0x1F)),
            opcode::LSHR  => binop_impl!(|x, y| x >> (y & 0x1F)),
            opcode::ASHR  => binop_impl!(|x, y| (x as i32) >> (y & 0x1F)),

            opcode::JAL => {
                let (rd, off) = decode_rc(inst);
                regs[rd] = pc;
                pc = u32::wrapping_add(pc, off << 2);
            }
            opcode::JALR => {
                let (rd, rs, off) = decode_rrc(inst);
                let new_pc = u32::wrapping_add(regs[rs], off) & !3;
                regs[rd] = pc;
                pc = new_pc;
            }

            opcode::LI => {
                let (rd, imm) = decode_rc(inst);
                regs[rd] = imm;
            }
            opcode::LUI => {
                let (rd, imm) = decode_rc(inst);
                regs[rd] = imm << 12;
            }

            opcode::MULW => {
                let (rd1, rd2, rs1, rs2) = decode_rrrr(inst);
                let lhs = regs[rs1] as i32 as u64; // Sign-extension.
                let rhs = regs[rs2] as i32 as u64; // Sign-extension.
                let mul = u64::wrapping_mul(lhs, rhs);
                regs[rd1] = mul as u32;
                regs[rd2] = (mul >> 32) as u32;
            }
            opcode::MULWU => {
                let (rd1, rd2, rs1, rs2) = decode_rrrr(inst);
                let lhs = regs[rs1] as u64; // Zero-extension.
                let rhs = regs[rs2] as u64; // Zero-extension.
                let mul = u64::wrapping_mul(lhs, rhs);
                regs[rd1] = mul as u32;
                regs[rd2] = (mul >> 32) as u32;
            }
            opcode::DIV => {
                let (rd1, rd2, rs1, rs2) = decode_rrrr(inst);
                let lhs = regs[rs1] as i32;
                let rhs = regs[rs2] as i32;
                let (q, r) = match rhs {
                    0 => (-1, lhs),
                    -1 => (i32::wrapping_neg(lhs), 0),
                    _ => (lhs / rhs, lhs % rhs),
                };
                regs[rd1] = q as u32;
                regs[rd2] = r as u32;
            }
            opcode::DIVU => {
                let (rd1, rd2, rs1, rs2) = decode_rrrr(inst);
                let lhs = regs[rs1];
                let rhs = regs[rs2];
                let (q, r) = match rhs {
                    0 => (!0, lhs),
                    _ => (lhs / rhs, lhs % rhs),
                };
                regs[rd1] = q;
                regs[rd2] = r;
            }

            opcode::SYSFN => {
                let (r, nr) = decode_rc(inst);
                match nr {
                    0 => vm_exit!(Ok(regs[r])),
                    1 => regs[r] = sysfn.read(),
                    2 => sysfn.write(regs[r]),
                    _ => vm_exit!(Err(Error::UnknownSysfn(nr))),
                }
            }

            _ => vm_exit!(Err(Error::UnknownInst(inst))),
        }
    }
}

fn decode_rc(inst: u32) -> (usize, u32) {
    let r1 = (inst >> 8) & 0xF;
    let imm = (inst as i32) >> 12;
    (r1 as usize, imm as u32)
}

fn decode_rrc(inst: u32) -> (usize, usize, u32) {
    let r1 = (inst >> 8) & 0xF;
    let r2 = (inst >> 12) & 0xF;
    let imm = (inst as i32) >> 16;
    (r1 as usize, r2 as usize, imm as u32)
}

fn decode_rrr(inst: u32) -> (usize, usize, usize) {
    let r1 = (inst >> 8) & 0xF;
    let r2 = (inst >> 12) & 0xF;
    let r3 = (inst >> 16) & 0xF;
    (r1 as usize, r2 as usize, r3 as usize)
}

fn decode_rrrr(inst: u32) -> (usize, usize, usize, usize) {
    let r1 = (inst >> 8) & 0xF;
    let r2 = (inst >> 12) & 0xF;
    let r3 = (inst >> 16) & 0xF;
    let r4 = (inst >> 20) & 0xF;
    (r1 as usize, r2 as usize, r3 as usize, r4 as usize)
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pc:  0x{:08X}", self.pc)?;
        writeln!(f, "x0:  0x{:08X}", self.regs[0])?;
        writeln!(f, "x1:  0x{:08X}", self.regs[1])?;
        writeln!(f, "x2:  0x{:08X}", self.regs[2])?;
        writeln!(f, "x3:  0x{:08X}", self.regs[3])?;
        writeln!(f, "x4:  0x{:08X}", self.regs[4])?;
        writeln!(f, "x5:  0x{:08X}", self.regs[5])?;
        writeln!(f, "x6:  0x{:08X}", self.regs[6])?;
        writeln!(f, "x7:  0x{:08X}", self.regs[7])?;
        writeln!(f, "x8:  0x{:08X}", self.regs[8])?;
        writeln!(f, "x9:  0x{:08X}", self.regs[9])?;
        writeln!(f, "x10: 0x{:08X}", self.regs[10])?;
        writeln!(f, "x11: 0x{:08X}", self.regs[11])?;
        writeln!(f, "x12: 0x{:08X}", self.regs[12])?;
        writeln!(f, "x13: 0x{:08X}", self.regs[13])?;
        writeln!(f, "x14: 0x{:08X}", self.regs[14])?;
        writeln!(f, "x15: 0x{:08X}", self.regs[15])?;
        Ok(())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            UnknownSysfn(nr) => {
                write!(f, "unknown system function {}", nr)
            }
            UnknownInst(inst) => {
                write!(f, "unknown instruction 0x{:08X}", inst)
            }
            InvalidAddr(addr) => {
                write!(f, "invalid address 0x{:X}", addr)
            }
            InvalidPc(pc) => {
                if pc % 4 != 0 {
                    write!(f, "misaligned program counter 0x{:X}", pc)
                } else {
                    write!(f, "invalid program counter 0x{:X}", pc)
                }
            }
        }
    }
}

impl error::Error for Error {}
