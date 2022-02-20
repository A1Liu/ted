use crate::compiler::*;
use crate::util::*;

pub type ERef = Ref<Expr>;

pub struct Ast {
    pub allocator: BucketList,
    pub block: Block,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    // translation from identifier to global memory numbering
    // pub scope: HashRef<'static, u32, u32>,
    pub stmts: &'static [Expr],
}

#[derive(Debug, Clone, Copy)]
pub struct Expr {
    pub kind: ExprKind,
    pub loc: CodeLoc,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind {
    Integer(u64),
    Ident {
        symbol: u32,
    },

    Call {
        callee: &'static Expr,
        args: &'static [Expr],
    },

    BinaryOp {
        kind: BinaryExprKind,
        left: &'static Expr,
        right: &'static Expr,
    },

    // TODO Eventually support:
    //
    // let a : int = 1
    // let a = 1
    // let a : int
    // let a
    Let {
        symbol: u32,
        value: &'static Expr,
    },

    Assign {
        symbol: u32,
        value: &'static Expr,
    },

    Block(Block),

    If {
        cond: &'static Expr,
        if_true: &'static Expr,
    },
    IfElse {
        cond: &'static Expr,
        if_true: &'static Expr,
        if_false: &'static Expr,
    },

    ForInfinite {
        block: Block,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryExprKind {
    Add,
    Multiply,
    Equal,
}
