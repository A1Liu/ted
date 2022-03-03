use crate::compiler::*;
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

pub fn check_ast(ast: &Ast) -> Result<TypeEnv, Error> {
    let mut types = TypeEnv {
        type_of: HashMap::new(),
        ident_to_expr: HashMap::new(),
    };

    let mut scope = ScopeEnv {
        vars: HashMap::new(),
    };

    let mut env = CheckEnv {
        types: &mut types,
        scope: &mut scope,
    };

    env.check_block(&ast.block)?;

    return Ok(types);
}

pub struct TypeEnv {
    pub type_of: HashMap<*const Expr, Type>,
    pub ident_to_expr: HashMap<*const Expr, *const Expr>,
}

// eventually this will be chaining
struct ScopeEnv {
    vars: HashMap<u32, &'static Expr>,
}

struct CheckEnv<'a> {
    types: &'a mut TypeEnv,
    scope: &'a mut ScopeEnv,
}

impl<'a> CheckEnv<'a> {
    fn chain<'b>(&'b mut self, scope: &'b mut ScopeEnv) -> CheckEnv<'b> {
        return CheckEnv {
            types: self.types,
            scope,
        };
    }

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

                if let Some(prev) = self.scope.vars.insert(symbol, value) {
                    return Err(Error::new("redeclared variable", expr.loc));
                }

                ty = Type::Null;
            }

            Ident { symbol } => {
                let value = self.scope.vars[&symbol] as *const Expr;

                self.types.ident_to_expr.insert(expr, value);

                ty = self.types.type_of[&value];
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

            Call { callee, args } => {
                for arg in args {
                    self.check_expr(arg)?;
                }

                ty = Type::Null;
            }

            k => unimplemented!("{:#?}", k),
        }

        if let Some(_) = self.types.type_of.insert(expr, ty) {
            panic!("idk");
        }

        return Ok(ty);
    }
}
