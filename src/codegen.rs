use crate::types::*;
fn push16() {
    println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき
}
fn pop16() {
    println!("      ldr x1, [sp], 16");
}

fn gen_addr(node: Node) {
    if let NodeKind::NdVar { var } = node.kind {
        let var = var.borrow();
        println!("      add x0, x29, {}", var.offset);
        return;
    }
}

fn gen_expr(node: Node) {
    if let NodeKind::NdNum { val } = node.kind {
        println!("      mov x0, {}", val);
        return;
    }

    if let NodeKind::NdVar { var } = node.kind {
        let var = var.borrow();
        println!("      add x0, x29, {}", var.offset); // えいや
        println!("      ldr x0, [x0]");
        return;
    }

    if let NodeKind::NdAdd { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      add x0, x1, x0");
        return;
    }

    if let NodeKind::NdSub { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      sub x0, x1, x0");
        return;
    }

    if let NodeKind::NdMul { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      mul x0, x1, x0");
        return;
    }

    if let NodeKind::NdDiv { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      sdiv x0, x1, x0");
        return;
    }

    if let NodeKind::NdNeg { lhs } = node.kind {
        gen_expr(*lhs);
        println!("      neg x0, x0");
        return;
    }

    if let NodeKind::NdEq { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x0, x1");
        println!("      cset x0, eq");
        return;
    }

    if let NodeKind::NdNe { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x0, x1");
        println!("      cset x0, ne");
        return;
    }

    if let NodeKind::NdLt { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x1, x0");
        println!("      cset x0, lt");
        return;
    }

    if let NodeKind::NdLe { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x1, x0");
        println!("      cset x0, le");
        return;
    }

    if let NodeKind::NdGt { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x1, x0");
        println!("      cset x0, gt");
        return;
    }

    if let NodeKind::NdGe { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      cmp x1, x0");
        println!("      cset x0, ge");
        return;
    }

    if let NodeKind::NdAssign { lhs, rhs } = node.kind {
        gen_addr(*lhs);
        push16();
        gen_expr(*rhs);
        pop16();
        println!("      str x0, [x1]");
        return;
    }
}

fn gen_stmt(node: Node) {
    if let NodeKind::NdExprStmt { lhs } = node.kind {
        gen_expr(*lhs);
        return;
    }

    if let NodeKind::NdReturn { lhs } = node.kind {
        gen_expr(*lhs);
        println!("	  b end");
        return;
    }

    if let NodeKind::NdBlock { body } = node.kind {
        for stmt in body {
            gen_stmt(stmt);
        }
        return;
    }
}

fn align16(i: isize) -> isize {
    (i + 15) & !15
}

pub fn codegen(ctx: Ctx) {
    let mut stack_size = 0;
    for var in ctx.variables {
        let mut var = var.borrow_mut();
        stack_size += var.offset;
        var.offset = var.offset * 16;
    }

    let prologue_size = align16(stack_size) + 16;
    println!(".text");
    println!(".global _main");
    println!("_main:");
    println!("      stp x29, x30, [sp, -{}]!", prologue_size);
    println!("      mov x29, sp");

    for stmt in ctx.body {
        // eprintln!("{:#?}", stmt);
        gen_stmt(stmt);
    }
    println!("end:");
    println!("      ldp x29, x30, [sp] ,{}", prologue_size);
    println!("      ret");
}
