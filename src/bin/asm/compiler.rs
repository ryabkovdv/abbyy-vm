use std::cmp;
use std::error;
use std::fmt;
use std::mem;

use my_vm::{binfile, opcode};
use smallvec::SmallVec;

use crate::ast::*;
use crate::id_table::{IdentTable, Symbol};
use crate::inst_syms::*;

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnknownInst,
    InvalidArgument,
    InvalidArgCount,

    UndefinedSymbol,
    RedefinedSymbol,

    AddrOverflow,
    ConstantTooLarge,
    TargetTooFar,
    MisalignedOffset,
}

#[derive(Clone, Default, Debug)]
pub struct Program {
    pub memory_size: u32,
    pub segments: Vec<Segment>,
}

#[derive(Clone, Default, Debug)]
pub struct Segment {
    pub addr: u32,
    pub data: Vec<u8>,
}

impl Program {
    pub fn new() -> Program {
        Default::default()
    }
}

impl Segment {
    pub fn new() -> Segment {
        Default::default()
    }
}

impl<'a> From<&'a Segment> for binfile::Segment<'a> {
    fn from(segment: &'a Segment) -> Self {
        binfile::Segment {
            addr: segment.addr,
            data: &segment.data,
        }
    }
}

pub fn compile(ast: &[Node], id_table: &IdentTable) -> Result<Program, Error> {
    let symtab = resolve_symbols(ast, id_table.len())?;
    let mut program = compile_tree(ast, &symtab)?;
    remove_empty_segments(&mut program.segments);

    Ok(program)
}

fn eval_expr(expr: &[Expr], symtab: &[Option<u32>]) -> Result<u32, ErrorKind> {
    let mut stack: SmallVec<[u32; 16]> = SmallVec::new();

    macro_rules! binop_impl {
        ($op:expr) => {{
            let y = stack.pop().unwrap();
            let x = stack.pop().unwrap();
            stack.push($op(x, y) as u32);
        }};
    }

    for op in expr {
        match op {
            Expr::Int(x) => stack.push(*x),
            Expr::Label(sym) => {
                stack.push(symtab[sym.id as usize].ok_or(ErrorKind::UndefinedSymbol)?);
            }
            Expr::Add  => binop_impl!(u32::wrapping_add),
            Expr::Sub  => binop_impl!(u32::wrapping_sub),
            Expr::Mul  => binop_impl!(u32::wrapping_mul),
            Expr::And  => binop_impl!(|x, y| x & y),
            Expr::Or   => binop_impl!(|x, y| x | y),
            Expr::Xor  => binop_impl!(|x, y| x ^ y),
            Expr::Shl  => binop_impl!(|x, y| x << (y & 0x1F)),
            Expr::Lshr => binop_impl!(|x, y| x >> (y & 0x1F)),
            Expr::Ashr => binop_impl!(|x, y| (x as i32) >> (y & 0x1F)),
        }
    }

    Ok(stack.pop().unwrap())
}

fn resolve_symbols(ast: &[Node], table_size: usize) -> Result<Vec<Option<u32>>, Error> {
    let mut symtab: Vec<Option<u32>> = vec![None; table_size];
    let mut addr: u32 = 0;

    for node in ast {
        macro_rules! err {
            ($e:expr) => {
                Err(Error { kind: $e, line: node.line })
            };
        }

        match node.kind {
            NodeKind::Label(sym) => {
                if symtab[sym.id as usize].is_some() {
                    return err!(ErrorKind::RedefinedSymbol);
                }
                symtab[sym.id as usize] = Some(addr);
            }
            NodeKind::Assign(sym) => {
                if node.args.len() != 1 {
                    return err!(ErrorKind::InvalidArgCount);
                }

                let value = match extract_and_eval_expr(&node.args[0], &symtab) {
                    Ok(value) => value,
                    Err(err) => return err!(err),
                };

                if symtab[sym.id as usize].is_some() {
                    return err!(ErrorKind::RedefinedSymbol);
                }
                symtab[sym.id as usize] = Some(value);
            }
            NodeKind::Inst(SEG) => {
                if node.args.len() != 1 {
                    return err!(ErrorKind::InvalidArgCount);
                }

                addr = match extract_and_eval_expr(&node.args[0], &symtab) {
                    Ok(value) => value,
                    Err(err) => return err!(err),
                };
            }
            NodeKind::Inst(t @ (D8 | D16 | D32)) => {
                if node.args.is_empty() {
                    return err!(ErrorKind::InvalidArgCount);
                }

                let size = match t {
                    D8 => 1,
                    D16 => 2,
                    D32 => 4,
                    _ => unreachable!(),
                };

                for arg in &node.args {
                    let offset = match arg {
                        Arg::Expr(_) => size,
                        Arg::Str(s) => {
                            if let Ok(value) = u32::try_from(s.len()) {
                                value
                            } else {
                                return err!(ErrorKind::AddrOverflow);
                            }
                        }
                        _ => return err!(ErrorKind::InvalidArgument),
                    };

                    if let Some(value) = addr.checked_add(offset) {
                        addr = value;
                    } else {
                        return err!(ErrorKind::AddrOverflow);
                    }
                }
            }
            _ => {
                if let Some(value) = addr.checked_add(4) {
                    addr = value;
                } else {
                    return err!(ErrorKind::AddrOverflow);
                }
            }
        }
    }

    Ok(symtab)
}

fn compile_tree(ast: &[Node], symtab: &[Option<u32>]) -> Result<Program, Error> {
    let mut segment = Segment::new();
    let mut program = Program::new();

    for node in ast {
        match compile_node(node, symtab, &mut program, &mut segment) {
            Ok(()) => {}
            Err(err) => return Err(Error { kind: err, line: node.line }),
        }
    }

    program.segments.push(segment);
    Ok(program)
}

fn compile_node(
    node: &Node,
    symtab: &[Option<u32>],
    program: &mut Program,
    segment: &mut Segment,
) -> Result<(), ErrorKind> {
    let eval_branch_offset = |target: u32| {
        let addr = segment.addr + (segment.data.len() as u32);
        let offset = target.wrapping_sub(addr).wrapping_sub(4);
        if offset % 4 == 0 {
            Ok(((offset as i32) >> 2) as u32)
        } else {
            Err(ErrorKind::MisalignedOffset)
        }
    };

    let inst = match node.kind {
        NodeKind::Inst(inst) => inst,
        _ => return Ok(()),
    };

    match inst {
        MEM => {
            check_arg_count(node.args.len(), 1)?;
            let new_memory_size = extract_and_eval_expr(&node.args[0], symtab)?;
            program.memory_size = cmp::max(program.memory_size, new_memory_size);
        }
        SEG => {
            program.segments.push(mem::replace(
                segment,
                Segment {
                    addr: extract_and_eval_expr(&node.args[0], symtab)?,
                    data: Vec::new(),
                },
            ));
        }
        D8 => {
            for arg in &node.args {
                match arg {
                    Arg::Expr(expr) => {
                        let value = match i8::try_from(eval_expr(expr, symtab)? as i32) {
                            Ok(value) => value,
                            Err(_) => return Err(ErrorKind::ConstantTooLarge),
                        };
                        segment.data.push(value as u8);
                    }
                    Arg::Str(s) => {
                        segment.data.extend_from_slice(s);
                    }
                    _ => return Err(ErrorKind::InvalidArgument),
                }
            }
        }
        D16 => {
            for arg in &node.args {
                let value = match i16::try_from(extract_and_eval_expr(arg, symtab)? as i32) {
                    Ok(value) => value,
                    Err(_) => return Err(ErrorKind::ConstantTooLarge),
                };
                segment.data.extend_from_slice(&i16::to_le_bytes(value));
            }
        }
        D32 => {
            for arg in &node.args {
                let value = extract_and_eval_expr(arg, symtab)?;
                segment.data.extend_from_slice(&u32::to_le_bytes(value));
            }
        }
        LI | LUI | SYSFN => {
            check_arg_count(node.args.len(), 2)?;

            let op = sym_to_opcode(inst);
            let r1 = extract_reg(&node.args[0])?;
            let imm = extract_and_eval_expr(&node.args[1], symtab)?;
            if inst == LUI {
                if (imm >> 20) != 0 {
                    return Err(ErrorKind::ConstantTooLarge);
                }
            } else {
                check_imm_fits(imm, 20)?;
            }

            segment.data.extend_from_slice(&encode_rc(op, r1, imm));
        }
        STS8 | STU8 | STS16 | STU16 | ST | LDS8 | LDU8 | LDS16 | LDU16 | LD | JALR | ADDI
        | RSUBI | MULI | ANDI | ORI | XORI | SHLI | LSHRI | ASHRI => {
            check_arg_count(node.args.len(), 3)?;

            let op = sym_to_opcode(inst);
            let r1 = extract_reg(&node.args[0])?;
            let r2 = extract_reg(&node.args[1])?;
            let imm = extract_and_eval_expr(&node.args[2], symtab)?;
            check_imm_fits(imm, 16)?;

            segment.data.extend_from_slice(&encode_rrc(op, r1, r2, imm));
        }
        ADD | SUB | MUL | AND | OR | XOR | SHL | LSHR | ASHR => {
            check_arg_count(node.args.len(), 3)?;

            let op = sym_to_opcode(inst);
            let r1 = extract_reg(&node.args[0])?;
            let r2 = extract_reg(&node.args[1])?;
            let r3 = extract_reg(&node.args[2])?;

            segment.data.extend_from_slice(&encode_rrr(op, r1, r2, r3));
        }
        MULW | MULWU | DIV | DIVU => {
            check_arg_count(node.args.len(), 4)?;

            let op = sym_to_opcode(inst);
            let r1 = extract_reg(&node.args[0])?;
            let r2 = extract_reg(&node.args[1])?;
            let r3 = extract_reg(&node.args[2])?;
            let r4 = extract_reg(&node.args[3])?;

            segment.data.extend_from_slice(&encode_rrrr(op, r1, r2, r3, r4));
        }
        BEQ | BNE | BLT | BGE | BLTU | BGEU | BGT | BLE | BGTU | BLEU => {
            check_arg_count(node.args.len(), 3)?;

            let op = sym_to_opcode(inst);
            let mut r1 = extract_reg(&node.args[0])?;
            let mut r2 = extract_reg(&node.args[1])?;
            if matches!(inst, BGT | BLE | BGTU | BLEU) {
                mem::swap(&mut r1, &mut r2);
            }

            let target = extract_and_eval_expr(&node.args[2], symtab)?;
            let imm = eval_branch_offset(target)?;
            check_imm_fits(imm, 16).map_err(|_| ErrorKind::TargetTooFar)?;

            segment.data.extend_from_slice(&encode_rrc(op, r1, r2, imm));
        }
        JAL | JMP | CALL => {
            let r1;
            let target;
            match inst {
                JAL => {
                    check_arg_count(node.args.len(), 2)?;
                    r1 = extract_reg(&node.args[0])?;
                    target = extract_expr(&node.args[1])?;
                }
                JMP => {
                    check_arg_count(node.args.len(), 1)?;
                    r1 = 0;
                    target = extract_expr(&node.args[0])?;
                }
                CALL => {
                    check_arg_count(node.args.len(), 1)?;
                    r1 = 1;
                    target = extract_expr(&node.args[0])?;
                }
                _ => unreachable!(),
            }

            let target = eval_expr(target, symtab)?;
            let imm = eval_branch_offset(target)?;
            check_imm_fits(imm, 20).map_err(|_| ErrorKind::TargetTooFar)?;

            segment.data.extend_from_slice(&encode_rc(opcode::JAL, r1, imm));
        }
        RET => {
            check_arg_count(node.args.len(), 0)?;

            segment.data.extend_from_slice(&encode_rrc(opcode::JALR, 0, 1, 0));
        }
        MOV => {
            check_arg_count(node.args.len(), 2)?;

            let r1 = extract_reg(&node.args[0])?;
            let r2 = extract_reg(&node.args[1])?;

            segment.data.extend_from_slice(&encode_rrr(opcode::ADDI, r1, r2, 0));
        }
        _ => return Err(ErrorKind::UnknownInst),
    }

    Ok(())
}

fn remove_empty_segments(segments: &mut Vec<Segment>) {
    let mut i = 0;
    while i < segments.len() {
        if segments[i].data.is_empty() {
            segments.swap_remove(i);
        } else {
            i += 1;
        }
    }
}

fn check_imm_fits(imm: u32, bits: u32) -> Result<(), ErrorKind> {
    let imm = imm as i32;
    let shift = 32 - bits;
    if (imm << shift) >> shift != imm {
        Err(ErrorKind::ConstantTooLarge)
    } else {
        Ok(())
    }
}

fn check_arg_count(got: usize, expected: usize) -> Result<(), ErrorKind> {
    if got != expected {
        Err(ErrorKind::InvalidArgCount)
    } else {
        Ok(())
    }
}

fn extract_reg(arg: &Arg) -> Result<u32, ErrorKind> {
    if let Arg::Reg(reg) = arg {
        Ok(*reg)
    } else {
        Err(ErrorKind::InvalidArgument)
    }
}

fn extract_expr(arg: &Arg) -> Result<&[Expr], ErrorKind> {
    if let Arg::Expr(expr) = arg {
        Ok(expr)
    } else {
        Err(ErrorKind::InvalidArgument)
    }
}

fn extract_and_eval_expr(arg: &Arg, symtab: &[Option<u32>]) -> Result<u32, ErrorKind> {
    if let Arg::Expr(expr) = arg {
        eval_expr(expr, symtab)
    } else {
        Err(ErrorKind::InvalidArgument)
    }
}

fn encode_rc(opcode: u32, r1: u32, imm: u32) -> [u8; 4] {
    u32::to_le_bytes(opcode | (r1 << 8) | (imm << 12))
}

fn encode_rrc(opcode: u32, r1: u32, r2: u32, imm: u32) -> [u8; 4] {
    u32::to_le_bytes(opcode | (r1 << 8) | (r2 << 12) | (imm << 16))
}

fn encode_rrr(opcode: u32, r1: u32, r2: u32, r3: u32) -> [u8; 4] {
    u32::to_le_bytes(opcode | (r1 << 8) | (r2 << 12) | (r3 << 16))
}

fn encode_rrrr(opcode: u32, r1: u32, r2: u32, r3: u32, r4: u32) -> [u8; 4] {
    u32::to_le_bytes(opcode | (r1 << 8) | (r2 << 12) | (r3 << 16) | (r4 << 20))
}

fn sym_to_opcode(sym: Symbol) -> u32 {
    match sym {
        LI    => opcode::LI,
        LUI   => opcode::LUI,
        SYSFN => opcode::SYSFN,
        STU8  => opcode::STU8,
        STU16 => opcode::STU16,
        ST    => opcode::ST,
        LDS8  => opcode::LDS8,
        LDU8  => opcode::LDU8,
        LDS16 => opcode::LDS16,
        LDU16 => opcode::LDU16,
        LD    => opcode::LD,
        JAL   => opcode::JAL,
        JALR  => opcode::JALR,
        BEQ   => opcode::BEQ,
        BNE   => opcode::BNE,
        BLT   => opcode::BLT,
        BGE   => opcode::BGE,
        BLTU  => opcode::BLTU,
        BGEU  => opcode::BGEU,
        ADDI  => opcode::ADDI,
        RSUBI => opcode::RSUBI,
        MULI  => opcode::MULI,
        ANDI  => opcode::ANDI,
        ORI   => opcode::ORI,
        XORI  => opcode::XORI,
        SHLI  => opcode::SHLI,
        LSHRI => opcode::LSHRI,
        ASHRI => opcode::ASHRI,
        ADD   => opcode::ADD,
        SUB   => opcode::SUB,
        MUL   => opcode::MUL,
        AND   => opcode::AND,
        OR    => opcode::OR,
        XOR   => opcode::XOR,
        SHL   => opcode::SHL,
        LSHR  => opcode::LSHR,
        ASHR  => opcode::ASHR,
        MULW  => opcode::MULW,
        MULWU => opcode::MULWU,
        DIV   => opcode::DIV,
        DIVU  => opcode::DIVU,

        STS8  => opcode::STU8,
        STS16 => opcode::STU16,
        BGT   => opcode::BLT,
        BLE   => opcode::BGE,
        BGTU  => opcode::BLTU,
        BLEU  => opcode::BGEU,
        JMP   => opcode::JAL,
        CALL  => opcode::JAL,
        RET   => opcode::JALR,
        MOV   => opcode::ADDI,

        _     => unreachable!(),
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl error::Error for Error {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        let msg = match self {
            UnknownInst      => "unknown instruction name",
            InvalidArgument  => "invalid argument type",
            InvalidArgCount  => "invalid argument count",
            UndefinedSymbol  => "undefined symbol",
            RedefinedSymbol  => "symbol redefined",
            AddrOverflow     => "address overflow",
            ConstantTooLarge => "constant is too large",
            TargetTooFar     => "branch target is too far",
            MisalignedOffset => "misaligned branch offset",
        };
        f.write_str(msg)
    }
}

impl error::Error for ErrorKind {}
