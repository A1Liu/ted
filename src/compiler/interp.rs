use crate::compiler::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

pub fn interpret(ast: &Ast) {
    let mut stack = BucketList::new();

    interp_block(stack.scoped(), &ast.block);
}

fn interp_block(mut alloc: ScopedBump, block: &Block) {
    for expr in block.stmts {
        interp(&mut alloc, expr);
    }
}

fn interp(alloc: &mut ScopedBump, e: &Expr) {
    use ExprKind::*;

    match e.kind {
        Let { value, .. } => {
            interp(alloc, value);
        }

        Block(block) => {
            interp_block(alloc.chain(), &block);
        }

        _ => unreachable!(),
    }
}
