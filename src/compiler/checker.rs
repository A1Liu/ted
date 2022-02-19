use crate::compiler::ast::*;
use crate::compiler::errors::*;
use crate::compiler::types::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

pub fn check_ast(ast: &Ast) -> Result<Pod<Op>, Error> {
    let mut ops = Pod::new();
    let mut env = TypeEnv {
        next_id: 0,
        scope: HashMap::new(),
    };

    env.check_block(&ast.block, &mut ops)?;

    return Ok(ops);
}

#[derive(Clone, Copy)]
struct VarInfo {
    value: OpResult,
}

struct TypeEnv {
    next_id: u32,
    scope: HashMap<u32, VarInfo>,
}

impl TypeEnv {
    fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        return id;
    }

    fn check_block(&mut self, block: &Block, ops: &mut Pod<Op>) -> Result<(), Error> {
        for expr in block.stmts {
            self.check_expr(expr, ops)?;
        }

        return Ok(());
    }

    fn check_expr(&mut self, expr: &Expr, ops: &mut Pod<Op>) -> Result<OpResult, Error> {
        use ExprKind::*;

        let id = self.id();
        let mut op = Op {
            kind: OpKind::Null { id },
            loc: expr.loc,
        };

        match expr.kind {
            Integer(value) => {
                op.kind = OpKind::Unsigned { id, value };
                ops.push(op);

                return Ok(op.kind.result());
            }

            Let { symbol, value } => {
                let result = self.check_expr(value, ops)?;

                let info = VarInfo { value: result };

                if let Some(prev) = self.scope.insert(symbol, info) {
                    return Err(Error::new("redeclared variable", op.loc));
                }

                return Ok(OpResult::Null);
            }

            BinaryOp { kind, left, right } => {
                let left = self.check_expr(left, ops)?;
                let right = self.check_expr(right, ops)?;

                let ty = left.ty();
                if ty != right.ty() {
                    return Err(Error::new(
                        "binary operation should be with values that are the same type",
                        expr.loc,
                    ));
                }

                if kind == BinaryExprKind::Add {
                    let result = OpResult::Value { id, ty };

                    op.kind = OpKind::Add {
                        result,
                        left,
                        right,
                    };

                    ops.push(op);

                    return Ok(result);
                }

                unreachable!();
            }

            _ => unreachable!(),
        }
    }

    fn check_binary(
        &mut self,
        op: BinaryExprKind,
        left: &Expr,
        right: &Expr,
    ) -> Result<OpResult, Error> {
        return Ok(OpResult::Null);
    }
}
