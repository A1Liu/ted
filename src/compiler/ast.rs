use crate::compiler::types::*;
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    // Means that the expression that returns this value doesn't ever return
    // a value directly (early return, loop forever, crash, ...)
    Never,

    // Void in C
    Null,

    Unsigned,
    String,
}

#[derive(Debug, Clone, Copy)]
pub struct Op {
    pub kind: OpKind,
    pub loc: CodeLoc,
}

#[derive(Debug, Clone, Copy)]
pub enum OpKind {
    Null {
        id: u32,
    },

    Unsigned {
        id: u32,
        value: u64,
    },

    Add {
        result: OpResult,
        left: OpResult,
        right: OpResult,
    },

    Print {
        input: u32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum OpResult {
    // Means that the expression that returns this value doesn't ever return
    // a value directly (early return, loop forever, crash, ...)
    Never,

    // Void in C
    Null,

    Value { id: u32, ty: Type },
}

impl OpKind {
    pub fn result(&self) -> OpResult {
        use OpKind::*;

        match *self {
            Null { id } => return OpResult::Value { id, ty: Type::Null },

            Unsigned { id, value } => {
                return OpResult::Value {
                    id,
                    ty: Type::Unsigned,
                }
            }

            Add { result, .. } => return result,

            Print { .. } => return OpResult::Null,
        }
    }
}

impl OpResult {
    pub fn ty(&self) -> Type {
        use OpResult::*;

        match *self {
            Never => return Type::Never,
            Null => return Type::Null,
            Value { id, ty } => return ty,
        }
    }
}
