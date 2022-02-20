use crate::compiler::ast::*;
use crate::compiler::errors::*;
use crate::compiler::types::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

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

pub fn check_ast(ast: &Ast) -> Result<(), Error> {
    let mut env = TypeEnv {
        type_of: HashMap::new(),
        scope: HashMap::new(),
    };

    env.check_block(&ast.block)?;

    return Ok(());
}

struct TypeEnv {
    type_of: HashMap<*const Expr, Type>,
    scope: HashMap<u32, &'static Expr>,
}

impl TypeEnv {
    fn check_block(&mut self, block: &Block) -> Result<Type, Error> {
        let mut ty = Type::Null;

        for expr in block.stmts {
            ty = self.check_expr(expr)?;
        }

        return Ok(ty);
    }

    fn check_expr(&mut self, expr: &'static Expr) -> Result<Type, Error> {
        use ExprKind::*;

        let mut ty;

        match expr.kind {
            Integer(value) => {
                ty = Type::Unsigned;
            }

            Let { symbol, value } => {
                let result = self.check_expr(value)?;

                if let Some(prev) = self.scope.insert(symbol, expr) {
                    return Err(Error::new("redeclared variable", expr.loc));
                }

                ty = Type::Null;
            }

            BinaryOp { kind, left, right } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;

                if left_ty != right_ty {
                    return Err(Error::new(
                        "binary operation should be with values that are the same type",
                        expr.loc,
                    ));
                }

                ty = left_ty;
            }

            _ => unreachable!(),
        }

        if let Some(_) = self.type_of.insert(expr, ty) {
            panic!("idk");
        }

        return Ok(ty);
    }
}
