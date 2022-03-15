use crate::id_table::{IdentTable, Symbol};

pub const MEM:   Symbol = Symbol { id: 0 };
pub const SEG:   Symbol = Symbol { id: 1 };

pub const D8:    Symbol = Symbol { id: 2 };
pub const D16:   Symbol = Symbol { id: 3 };
pub const D32:   Symbol = Symbol { id: 4 };

pub const LI:    Symbol = Symbol { id: 5 };
pub const LUI:   Symbol = Symbol { id: 6 };
pub const SYSFN: Symbol = Symbol { id: 7 };

pub const STU8:  Symbol = Symbol { id: 8 };
pub const STU16: Symbol = Symbol { id: 9 };
pub const ST:    Symbol = Symbol { id: 10 };

pub const LDS8:  Symbol = Symbol { id: 11 };
pub const LDU8:  Symbol = Symbol { id: 12 };
pub const LDS16: Symbol = Symbol { id: 13 };
pub const LDU16: Symbol = Symbol { id: 14 };
pub const LD:    Symbol = Symbol { id: 15 };

pub const JAL:   Symbol = Symbol { id: 16 };
pub const JALR:  Symbol = Symbol { id: 17 };
pub const BEQ:   Symbol = Symbol { id: 18 };
pub const BNE:   Symbol = Symbol { id: 19 };
pub const BLT:   Symbol = Symbol { id: 20 };
pub const BGE:   Symbol = Symbol { id: 21 };
pub const BLTU:  Symbol = Symbol { id: 22 };
pub const BGEU:  Symbol = Symbol { id: 23 };

pub const ADDI:  Symbol = Symbol { id: 24 };
pub const RSUBI: Symbol = Symbol { id: 25 };
pub const MULI:  Symbol = Symbol { id: 26 };
pub const ANDI:  Symbol = Symbol { id: 27 };
pub const ORI:   Symbol = Symbol { id: 28 };
pub const XORI:  Symbol = Symbol { id: 29 };
pub const SHLI:  Symbol = Symbol { id: 30 };
pub const LSHRI: Symbol = Symbol { id: 31 };
pub const ASHRI: Symbol = Symbol { id: 32 };

pub const ADD:   Symbol = Symbol { id: 33 };
pub const SUB:   Symbol = Symbol { id: 34 };
pub const MUL:   Symbol = Symbol { id: 35 };
pub const AND:   Symbol = Symbol { id: 36 };
pub const OR:    Symbol = Symbol { id: 37 };
pub const XOR:   Symbol = Symbol { id: 38 };
pub const SHL:   Symbol = Symbol { id: 39 };
pub const LSHR:  Symbol = Symbol { id: 40 };
pub const ASHR:  Symbol = Symbol { id: 41 };

pub const MULW:  Symbol = Symbol { id: 42 };
pub const MULWU: Symbol = Symbol { id: 43 };
pub const DIV:   Symbol = Symbol { id: 44 };
pub const DIVU:  Symbol = Symbol { id: 45 };

pub const STS8:  Symbol = Symbol { id: 46 };
pub const STS16: Symbol = Symbol { id: 47 };
pub const BGT:   Symbol = Symbol { id: 48 };
pub const BLE:   Symbol = Symbol { id: 49 };
pub const BGTU:  Symbol = Symbol { id: 50 };
pub const BLEU:  Symbol = Symbol { id: 51 };
pub const JMP:   Symbol = Symbol { id: 52 };
pub const CALL:  Symbol = Symbol { id: 53 };
pub const RET:   Symbol = Symbol { id: 54 };
pub const MOV:   Symbol = Symbol { id: 55 };

pub fn make_proper_id_table() -> IdentTable {
    let mut id_table = IdentTable::new();

    id_table.insert("mem");
    id_table.insert("seg");
    id_table.insert("d8");
    id_table.insert("d16");
    id_table.insert("d32");
    id_table.insert("li");
    id_table.insert("lui");
    id_table.insert("sysfn");
    id_table.insert("st.u8");
    id_table.insert("st.u16");
    id_table.insert("st");
    id_table.insert("ld.s8");
    id_table.insert("ld.u8");
    id_table.insert("ld.s16");
    id_table.insert("ld.u16");
    id_table.insert("ld");
    id_table.insert("jal");
    id_table.insert("jalr");
    id_table.insert("beq");
    id_table.insert("bne");
    id_table.insert("blt");
    id_table.insert("bge");
    id_table.insert("bltu");
    id_table.insert("bgeu");
    id_table.insert("addi");
    id_table.insert("rsubi");
    id_table.insert("muli");
    id_table.insert("andi");
    id_table.insert("ori");
    id_table.insert("xori");
    id_table.insert("shli");
    id_table.insert("lshri");
    id_table.insert("ashri");
    id_table.insert("add");
    id_table.insert("sub");
    id_table.insert("mul");
    id_table.insert("and");
    id_table.insert("or");
    id_table.insert("xor");
    id_table.insert("shl");
    id_table.insert("lshr");
    id_table.insert("ashr");
    id_table.insert("mulw");
    id_table.insert("mulwu");
    id_table.insert("div");
    id_table.insert("divu");

    id_table.insert("st.s8");
    id_table.insert("st.s16");
    id_table.insert("bgt");
    id_table.insert("ble");
    id_table.insert("bgtu");
    id_table.insert("bleu");
    id_table.insert("jmp");
    id_table.insert("call");
    id_table.insert("ret");
    id_table.insert("mov");

    id_table
}
