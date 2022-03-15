use std::error;
use std::fmt;

use smallvec::{Array, SmallVec};

use crate::ast::*;
use crate::id_table::IdentTable;
use crate::lexer::{self, Lexer, Token};

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    JunkInLine,
    MissingRegName,
    InvalidRegName,
    InvalidIntLiteral,
    MissingClosingParen,
    ExpectedExpr,
    LexerError(lexer::Error),
}

pub fn parse(
    lexer: &mut Lexer,
    id_table: &mut IdentTable,
    ast: &mut Vec<Node>,
) -> Result<(), Error> {
    let mut line = 1;

    macro_rules! err {
        ($e:expr) => {
            Err(Error { kind: $e, line })
        };
    }

    loop {
        if let Token::Label(s) = lexer.peek() {
            lexer.next();

            ast.push(Node {
                kind: NodeKind::Label(id_table.insert(s)),
                args: SmallVec::new(),
                line,
            });
        }

        if let Token::Ident(s) = lexer.peek() {
            lexer.next();

            let sym = id_table.insert(s);

            let kind;
            if lexer.peek() == Token::Equal {
                lexer.next();
                kind = NodeKind::Assign(sym);
            } else {
                kind = NodeKind::Inst(sym);
            }

            let mut args = SmallVec::new();
            match parse_args(lexer, id_table, &mut args) {
                Ok(()) => {}
                Err(err) => return err!(err),
            }
            ast.push(Node { kind, args, line });
        }

        match lexer.next() {
            Token::Eol => {}
            Token::Eof => return Ok(()),
            Token::Err(err) => return err!(ErrorKind::LexerError(err)),
            _ => return err!(ErrorKind::JunkInLine),
        }
        line += 1;
    }
}

fn parse_args<A>(
    lexer: &mut Lexer,
    id_table: &mut IdentTable,
    args: &mut SmallVec<A>,
) -> Result<(), ErrorKind>
where
    A: Array<Item = Arg>,
{
    if matches!(lexer.peek(), Token::Eof | Token::Eol) {
        return Ok(());
    }

    loop {
        args.push(match lexer.peek() {
            Token::Reg(s) => {
                lexer.next();
                Arg::Reg(parse_reg(s)?)
            }
            Token::Str(s) => {
                lexer.next();
                Arg::Str(SmallVec::from_slice(s.as_bytes()))
            }
            _ => {
                let mut expr = SmallVec::new();
                parse_expr(lexer, id_table, &mut expr)?;
                Arg::Expr(expr)
            }
        });

        if lexer.peek() != Token::Comma {
            return Ok(());
        }
        lexer.next();
    }
}

fn parse_expr<A>(
    lexer: &mut Lexer,
    id_table: &mut IdentTable,
    expr: &mut SmallVec<A>,
) -> Result<(), ErrorKind>
where
    A: Array<Item = Expr>,
{
    parse_primary_expr(lexer, id_table, expr)?;
    if precedence(lexer.peek()) > 0 {
        parse_expr_helper(lexer, id_table, expr, 0)?;
    }
    Ok(())
}

fn parse_expr_helper<A>(
    lexer: &mut Lexer,
    id_table: &mut IdentTable,
    expr: &mut SmallVec<A>,
    min_prec: u32,
) -> Result<(), ErrorKind>
where
    A: Array<Item = Expr>,
{
    let mut lookahead = lexer.peek();
    loop {
        let op = lookahead;
        let max_prec = precedence(op);

        lexer.next();
        parse_primary_expr(lexer, id_table, expr)?;
        lookahead = lexer.peek();

        while precedence(lookahead) > max_prec {
            parse_expr_helper(lexer, id_table, expr, max_prec)?;
            lookahead = lexer.peek();
        }

        expr.push(token_to_binop(op));

        if precedence(lookahead) <= min_prec {
            return Ok(());
        }
    }
}

fn parse_primary_expr<A>(
    lexer: &mut Lexer,
    id_table: &mut IdentTable,
    expr: &mut SmallVec<A>,
) -> Result<(), ErrorKind>
where
    A: Array<Item = Expr>,
{
    match lexer.peek() {
        Token::Lparen => {
            lexer.next();
            parse_expr(lexer, id_table, expr)?;
            if lexer.next() != Token::Rparen {
                return Err(ErrorKind::MissingClosingParen);
            }
            Ok(())
        }
        t @ (Token::Add | Token::Sub | Token::Xor) => {
            lexer.next();

            match t {
                Token::Add => {}
                Token::Sub => expr.push(Expr::Int(0)),
                Token::Xor => expr.push(Expr::Int(!0)),
                _ => unreachable!(),
            }

            parse_primary_expr(lexer, id_table, expr)?;

            match t {
                Token::Add => {}
                Token::Sub => expr.push(Expr::Sub),
                Token::Xor => expr.push(Expr::Xor),
                _ => unreachable!(),
            }

            Ok(())
        }
        Token::Char(ch) => {
            lexer.next();
            expr.push(Expr::Int(ch as u32));
            Ok(())
        }
        Token::Int(s) => {
            lexer.next();
            if let Ok(value) = parse_int(s) {
                expr.push(Expr::Int(value));
            } else {
                return Err(ErrorKind::InvalidIntLiteral);
            }
            Ok(())
        }
        Token::Ident(s) => {
            lexer.next();
            expr.push(Expr::Label(id_table.insert(s)));
            Ok(())
        }
        Token::Err(err) => Err(ErrorKind::LexerError(err)),
        _ => Err(ErrorKind::ExpectedExpr),
    }
}

fn precedence(token: Token) -> u32 {
    match token {
        Token::Shl | Token::Lshr | Token::Ashr => 3,
        Token::And | Token::Mul => 2,
        Token::Add | Token::Sub | Token::Or | Token::Xor => 1,
        _ => 0,
    }
}

fn token_to_binop(token: Token) -> Expr {
    match token {
        Token::Add  => Expr::Add,
        Token::Sub  => Expr::Sub,
        Token::And  => Expr::And,
        Token::Mul  => Expr::Mul,
        Token::Or   => Expr::Or,
        Token::Xor  => Expr::Xor,
        Token::Shl  => Expr::Shl,
        Token::Lshr => Expr::Lshr,
        Token::Ashr => Expr::Ashr,
        _ => unreachable!(),
    }
}

fn parse_reg(s: &str) -> Result<u32, ErrorKind> {
    match s {
        "x0"  | "zero" => Ok(0),
        "x1"  | "lr"   => Ok(1),
        "x2"  | "sp"   => Ok(2),
        "x3"  | "a0"   => Ok(3),
        "x4"  | "a1"   => Ok(4),
        "x5"  | "a2"   => Ok(5),
        "x6"  | "a3"   => Ok(6),
        "x7"  | "a4"   => Ok(7),
        "x8"  | "a5"   => Ok(8),
        "x9"  | "s0"   => Ok(9),
        "x10" | "s1"   => Ok(10),
        "x11" | "s2"   => Ok(11),
        "x12" | "s3"   => Ok(12),
        "x13" | "s4"   => Ok(13),
        "x14" | "s5"   => Ok(14),
        "x15" | "s6"   => Ok(15),

        "" => Err(ErrorKind::MissingRegName),
        _  => Err(ErrorKind::InvalidRegName),
    }
}

fn parse_int(s: &str) -> Result<u32, ErrorKind> {
    macro_rules! parse_int {
        ($s:expr, $base:expr) => {
            if let Ok(value) = u32::from_str_radix($s, $base) {
                Ok(value)
            } else {
                return Err(ErrorKind::InvalidIntLiteral);
            }
        };
    }

    let bytes = s.as_bytes();
    match s.len() {
        0 => unreachable!(),
        1 => Ok((bytes[0] - b'0') as u32),
        2 => {
            if matches!(bytes[1], b'0'..=b'9') {
                Ok(((bytes[0] - b'0') * 10 + (bytes[1] - b'0')) as u32)
            } else {
                Err(ErrorKind::InvalidIntLiteral)
            }
        }
        _ => {
            if let Some(hex) = s.strip_prefix("0x").or(s.strip_prefix("0X")) {
                parse_int!(hex, 16)
            } else {
                parse_int!(s, 10)
            }
        }
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
            JunkInLine          => "junk at the end of a line",
            MissingRegName      => "missing register name",
            InvalidRegName      => "invalid register name",
            InvalidIntLiteral   => "invalid integer literal",
            MissingClosingParen => "missing closing ')'",
            ExpectedExpr        => "expected expression",
            LexerError(err)     => return err.fmt(f),
        };
        f.write_str(msg)
    }
}

impl error::Error for ErrorKind {}
