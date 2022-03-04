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
        parent: None,
    };

    let mut env = CheckEnv {
        types: &mut types,
        scope,
    };

    for expr in ast.block.stmts {
        env.check_expr(expr)?;
    }

    return Ok(types);
}

pub struct TypeEnv {
    pub type_of: HashMap<*const Expr, Type>,
    pub ident_to_expr: HashMap<*const Expr, *const Expr>,
}

struct CheckEnv<'a> {
    types: &'a mut TypeEnv,
    scope: ScopeEnv<'a>,
}

impl<'a> CheckEnv<'a> {
    fn chain<'b>(&'b mut self) -> CheckEnv<'b> {
        return CheckEnv {
            types: self.types,
            scope: ScopeEnv {
                parent: Some(&mut self.scope),
                vars: HashMap::new(),
            },
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

        println!("{:?}", expr.kind);
        for (k, _) in self.scope.vars.iter() {
            println!("{:?}", k);
        }

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
                let value = match self.scope.search(symbol) {
                    Some(e) => e as *const Expr,
                    None => {
                        return Err(Error::new("couldn't find variable", expr.loc));
                    }
                };

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

            Block(block) => {
                let mut child = self.chain();
                println!("wot");

                for expr in block.stmts {
                    child.check_expr(expr)?;
                }

                ty = Type::Null;
            }

            Call { callee, args } => {
                const PRINT: u32 = Key::Print as u32;

                match callee.kind {
                    Ident { symbol: PRINT } => {}
                    _ => {
                        unimplemented!("function calls besides print aren't implemented");
                    }
                }

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

// eventually this will be chaining
struct ScopeEnv<'a> {
    parent: Option<&'a ScopeEnv<'a>>,
    vars: HashMap<u32, &'static Expr>,
}

impl<'a> ScopeEnv<'a> {
    fn search(&self, symbol: u32) -> Option<&'static Expr> {
        let mut current = self;

        loop {
            if let Some(e) = current.vars.get(&symbol) {
                println!("asdf");
                return Some(*e);
            }

            println!("Hellope");

            if let Some(parent) = current.parent {
                current = parent;

                println!("Hello");

                continue;
            }

            return None;
        }
    }
}
