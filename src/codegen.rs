use crate::types::*;
fn push16() {
    println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき
}
fn pop16() {
    println!("      ldr x1, [sp], 16");
}

static mut IFIDX: usize = 0;
static mut FORIDX: usize = 0;

fn load(ty: &Type) {
    if let TypeKind::TyArray { .. } = ty.kind {
        return;
    }
    println!("  ldr x0, [x0]");
}

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
        load(&var.ty);
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
        load(node.ty.as_ref().unwrap()); // 正しいか？
        return;
    }

    if let NodeKind::NdFuncCall { name, args } = node.kind {
        for arg in &args {
            gen_expr(arg.clone());
            push16();
        }
        for i in (0..args.len()).rev() {
            println!("      ldr x{}, [sp], 16", i);
        }
        println!("      bl _{}", name);
        return;
    }

    panic!("not expected node: {:#?}", node);
}

fn gen_stmt(node: Node, funcname: &str) {
    if let NodeKind::NdExprStmt { lhs } = node.kind {
        gen_expr(*lhs);
        return;
    }

    if let NodeKind::NdReturn { lhs } = node.kind {
        gen_expr(*lhs);
        println!("      b end.{}", funcname);
        return;
    }

    if let NodeKind::NdBlock { body } = node.kind {
        for stmt in body {
            gen_stmt(stmt, funcname);
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
            gen_stmt(*then, funcname);
            println!("	  b endif.{}", idx);
            println!("else.{}:", idx);
            gen_stmt(*els, funcname);
        } else {
            println!("	  b.ne endif.{}", idx);
            gen_stmt(*then, funcname);
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
        gen_stmt(*init, funcname);
        println!("	  b cond.{}", idx);
        println!("startfor.{}:", idx);
        gen_stmt(*body, funcname);
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
        gen_stmt(*body, funcname);
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
    for (name, func) in &ctx.functions {
        let mut stack_size = 16; // 少々余分に撮っていると思う

        // 各関数の変数に対してスタックサイズを計算
        // どうやってアラインメントすれば良いのか正直わからん
        for var in &func.variables {
            let mut var = var.borrow_mut();
            var.offset = stack_size;
            stack_size += var.ty.size as isize;
        }
        stack_size = align16(stack_size);

        // 関数名に基づくラベル
        println!(".text");
        println!(".global _{}", name);
        println!("_{}:", name);
        println!("      stp x29, x30, [sp, -{}]!", stack_size);
        println!("      mov x29, sp");

        // 引数の処理
        for (i, arg) in func.args.iter().enumerate() {
            // 他のアドレスを計算する際、x0を使うので、最初の引数のみ特別扱いして対比する
            if i == 0 {
                println!("      mov x9, x0");
                gen_addr(arg.clone()); // x0にアドレスが入る
                println!("      str x9, [x0]");
                continue;
            }
            gen_addr(arg.clone()); // x0にアドレスが入る
            println!("      str x{}, [x0]", i);
        }

        // 関数のbodyに対して`gen_stmt`を呼び出す
        if let Some(body) = &func.body {
            gen_stmt(body.clone(), name);
        }

        println!("end.{}:", name);
        println!("      ldp x29, x30, [sp] ,{}", stack_size);
        println!("      ret");
    }
}
