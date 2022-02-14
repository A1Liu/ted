use crate::compiler::types::*;
use crate::util::*;

pub struct Ast {
    pub allocator: BucketList,
    pub block: Block,
}

#[derive(Clone, Copy)]
pub struct Block {
    // translation from identifier to memory numbering
    pub scope: HashRef<'static, u32, u32>,
    pub stmts: &'static [Stmt],
}

#[derive(Clone, Copy)]
pub struct Stmt {
    kind: StmtKind,
    loc: CodeLoc,
}

#[derive(Clone, Copy)]
pub struct Expr {
    kind: ExprKind,
    loc: CodeLoc,
}

#[derive(Clone, Copy)]
pub enum StmtKind {
    Assign {},
    For {
        iter: u32,
        index: u32,
        expression: &'static Expr,
    },
}

#[derive(Clone, Copy)]
pub enum ExprKind {
    Unsigned(u64),
    Signed(i64),
    Ident(u32),
}
