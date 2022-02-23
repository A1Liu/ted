use crate::compiler::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

pub fn interpret(ast: &Ast) {
    let mut stack = BucketList::new();

    let mut interp = Interp {
        type_of: HashMap::new(),
    };

    let mut values = HashMap::new();

    let scope = Scope {
        values: &mut values,
        alloc: stack.scoped(),
    };

    interp.block(scope, &ast.block);
}

struct Interp {
    type_of: HashMap<*const Expr, Type>,
}

impl Interp {
    fn block(&mut self, mut scope: Scope, block: &Block) {
        for expr in block.stmts {
            self.expr(&mut scope, expr);
        }
    }

    fn expr(&mut self, scope: &mut Scope, e: &Expr) -> Register {
        use ExprKind::*;

        match e.kind {
            Let { value, .. } => {
                let value = self.expr(scope, value);

                return ZERO;
            }

            Block(block) => {
                self.block(scope.chain(), &block);

                return ZERO;
            }

            _ => unreachable!(),
        }
    }
}

struct Scope<'a> {
    values: &'a mut HashMap<*const Expr, u64>,
    alloc: ScopedBump<'a>,
}

impl<'a> Scope<'a> {
    fn chain<'b>(&'b mut self) -> Scope<'b> {
        return Scope {
            values: self.values,
            alloc: self.alloc.chain(),
        };
    }
}

const ZERO: Register = Register { value: 0 };

#[derive(Clone, Copy)]
struct Register {
    value: u64,
}

impl Register {
    fn u32(&self) -> u32 {
        return self.value as u32;
    }

    fn u64(&self) -> u64 {
        return self.value;
    }
}
