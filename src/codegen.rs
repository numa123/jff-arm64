use crate::types::*;
fn push16() {
    println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき
}
fn pop16() {
    println!("      ldr x1, [sp], 16");
}

static mut IFIDX: usize = 0;
static mut FORIDX: usize = 0;

fn gen_addr(node: Node) {
    if let NodeKind::NdVar { var } = node.kind {
        let var = var.borrow();
        println!("      add x0, x29, {}", var.offset);
        return;
    }
    if let NodeKind::NdDeref { lhs } = node.kind {
        gen_expr(*lhs);
        return;
    }
    panic!("not expected node: {:#?}", node);
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

    if let NodeKind::NdAddr { lhs } = node.kind {
        gen_addr(*lhs);
        return;
    }

    if let NodeKind::NdDeref { lhs } = node.kind {
        gen_expr(*lhs);
        println!("      ldr x0, [x0]");
        return;
    }

    if let NodeKind::NdFuncCall { name } = node.kind {
        println!("      bl _{}", name);
        return;
    }

    panic!("not expected node: {:#?}", node);
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

    // あえて冗長なアセンブリを出力
    if let NodeKind::NdIf { cond, then, els } = node.kind {
        let idx = unsafe { IFIDX };
        unsafe { IFIDX += 1 };
        gen_expr(*cond);
        println!("	  cmp x0, 1");
        if let Some(els) = els {
            println!("	  b.ne else.{}", idx);
            gen_stmt(*then);
            println!("	  b endif.{}", idx);
            println!("else.{}:", idx);
            gen_stmt(*els);
        } else {
            println!("	  b.ne endif.{}", idx);
            gen_stmt(*then);
        }
        println!("endif.{}:", idx);
        return;
    }

    if let NodeKind::NdFor {
        init,
        cond,
        inc,
        body,
    } = node.kind
    {
        let idx = unsafe { FORIDX };
        unsafe { FORIDX += 1 };
        gen_stmt(*init);
        println!("	  b cond.{}", idx);
        println!("startfor.{}:", idx);
        gen_stmt(*body);
        if let Some(inc) = inc {
            gen_expr(*inc);
        }
        println!("cond.{}:", idx);
        if let Some(cond) = cond {
            gen_expr(*cond);
            println!("	  cmp x0, 1");
            println!("	  b.ne endfor.{}", idx);
        }
        println!("	  b startfor.{}", idx);
        println!("endfor.{}:", idx);
        return;
    }

    if let NodeKind::NdWhile { cond, body } = node.kind {
        let idx = unsafe { FORIDX };
        unsafe { FORIDX += 1 };
        println!("startwhile.{}:", idx);
        gen_expr(*cond);
        println!("	  cmp x0, 1");
        println!("	  b.ne endwhile.{}", idx);
        gen_stmt(*body);
        println!("	  b startwhile.{}", idx);
        println!("endwhile.{}:", idx);
        return;
    }

    panic!("not expected node"); // matchにした方が良い
}

fn align16(i: isize) -> isize {
    (i + 15) & !15
}

pub fn codegen(ctx: Ctx) {
    let mut stack_size = 0;
    for var in ctx.variables {
        let mut var = var.borrow_mut();
        stack_size += var.offset;
        var.offset = var.offset + 16;
    }

    let prologue_size = align16(stack_size) + 16;
    println!(".text");
    println!(".global _main");
    println!("_main:");
    println!("      stp x29, x30, [sp, -{}]!", prologue_size);
    println!("      mov x29, sp");

    for stmt in ctx.body {
        gen_stmt(stmt);
    }
    println!("end:");
    println!("      ldp x29, x30, [sp] ,{}", prologue_size);
    println!("      ret");
}
