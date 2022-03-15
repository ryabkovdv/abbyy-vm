use smallvec::SmallVec;

use crate::id_table::Symbol;

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub args: SmallVec<[Arg; 3]>,
    pub line: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum NodeKind {
    Label(Symbol),
    Assign(Symbol),
    Inst(Symbol),
}

#[derive(Clone, Debug)]
pub enum Arg {
    Reg(u32),
    Str(SmallVec<[u8; 16]>),
    Expr(SmallVec<[Expr; 2]>),
}

#[derive(Clone, Copy, Debug)]
pub enum Expr {
    Int(u32),
    Label(Symbol),

    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Shl,
    Lshr,
    Ashr,
}
