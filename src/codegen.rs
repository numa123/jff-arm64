use crate::types::*;
fn push16() {
    println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき?
}
fn pop16() {
    println!("      ldr x1, [sp], 16 // pop");
}

static mut IFIDX: usize = 0;
static mut FORIDX: usize = 0;
static mut CURRENTFN: String = String::new();

fn load(ty: &Type) {
    match ty.kind {
        TypeKind::TyArray { .. } => {
            return;
        }
        TypeKind::TyChar => {
            println!("      ldrsb x0, [x0]");
        }
        _ => {
            println!("      ldr x0, [x0]");
        }
    }
}

fn store(ty: &Type) {
    if ty.size == 1 {
        println!("      strb w0, [x1]");
    } else {
        println!("      str x0, [x1]");
    }
    // 4だったらwとかね 今の8を4にしてintにしたら良いかも
}

fn gen_addr(node: Node) {
    match node.kind {
        NodeKind::NdVar { var } => {
            let var = var.borrow();
            if var.is_local {
                println!("      add x0, x29, {}", var.offset);
            } else {
                println!("      adrp x0, {}@PAGE", var.name); // what is PAGE?
                println!("      add x0, x0, {}@PAGEOFF;", var.name);
            }
            return;
        }
        NodeKind::NdDeref { lhs, .. } => {
            gen_expr(*lhs);
            return;
        }
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::NdNum { val } => {
            println!("      mov x0, {}", val);
            return;
        }
        NodeKind::NdVar { ref var } => {
            let ty = var.borrow().ty.clone();
            gen_addr(node);
            load(&ty);
            return;
        }
        NodeKind::NdAdd { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      add x0, x1, x0");
            return;
        }
        NodeKind::NdSub { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sub x0, x1, x0");
            return;
        }
        NodeKind::NdMul { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      mul x0, x1, x0");
            return;
        }
        NodeKind::NdDiv { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sdiv x0, x1, x0");
            return;
        }
        NodeKind::NdMod { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sdiv x2, x1, x0");
            println!("      msub x0, x2, x0, x1");
            return;
        }
        NodeKind::NdNeg { lhs } => {
            gen_expr(*lhs);
            println!("      neg x0, x0");
            return;
        }
        NodeKind::NdEq { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x0, x1");
            println!("      cset x0, eq");
            return;
        }
        NodeKind::NdNe { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x0, x1");
            println!("      cset x0, ne");
            return;
        }
        NodeKind::NdLt { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, lt");
            return;
        }
        NodeKind::NdLe { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, le");
            return;
        }
        NodeKind::NdGt { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, gt");
            return;
        }
        NodeKind::NdGe { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, ge");
            return;
        }
        NodeKind::NdAnd { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("  mov x2, 0");
            println!("  cmp x1, 0");
            println!("  cset x2, ne");
            println!("  cmp x0, 0");
            println!("  cset x0, ne");
            println!("  and x0, x0, x2");
            return;
        }
        NodeKind::NdOr { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("  mov x2, 0");
            println!("  cmp x1, 0");
            println!("  cset x2, ne");
            println!("  cmp x0, 0");
            println!("  cset x0, ne");
            println!("  orr x0, x0, x2");
            return;
        }
        NodeKind::NdAssign { lhs, rhs } => {
            gen_addr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            store(node.ty.as_ref().unwrap()); // unwrap使わずにいけないかな
            return;
        }
        NodeKind::NdAddr { lhs } => {
            gen_addr(*lhs);
            return;
        }
        NodeKind::NdDeref { lhs, .. } => {
            gen_expr(*lhs);
            load(node.ty.as_ref().unwrap()); // 正しいか？
            return;
        }
        NodeKind::NdFuncCall { name, args } => {
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
        NodeKind::NdGNUStmtExpr { body } => {
            for stmt in body {
                gen_stmt(stmt);
            }
            return;
        }
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn gen_stmt(node: Node) {
    match node.kind {
        NodeKind::NdExprStmt { lhs } => {
            gen_expr(*lhs);
            return;
        }
        NodeKind::NdReturn { lhs } => {
            gen_expr(*lhs);
            println!("      b end.{}", unsafe { CURRENTFN.clone() });
            return;
        }
        NodeKind::NdBlock { body } => {
            for stmt in body {
                gen_stmt(stmt);
            }
            return;
        }
        NodeKind::NdIf { cond, then, els } => {
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
        NodeKind::NdFor {
            init,
            cond,
            inc,
            body,
        } => {
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
        NodeKind::NdWhile { cond, body } => {
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
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn align16(i: isize) -> isize {
    (i + 15) & !15
}

pub fn codegen(ctx: Ctx) {
    for var in &ctx.global_variables {
        let var = var.borrow();
        match &var.init_gval {
            Some(gval) => {
                match gval {
                    InitGval::Str(s) => {
                        let trimmed = s.trim_end_matches('\0'); // ヌル文字を除去
                        println!(".text");
                        println!(".cstring"); // セクションの指定
                        println!(".align 3"); // ポインタは8byte。align 3 は　2^3 = 8byteでアラインメント
                        println!("{}:", var.name);
                        if trimmed.len() == 0 {
                            println!("      .asciz \"\"");
                        } else {
                            println!("      .asciz \"{}\"", trimmed);
                        }
                    }
                    InitGval::Num(val) => {
                        println!(".data");
                        println!(".global {}", var.name);
                        println!("{}:", var.name);
                        println!("      .xword {}", val);
                    }
                }
            }
            None => {
                println!(".data");
                println!(".global {}", var.name);
                println!("{}:", var.name);
                println!("      .zero {}", var.ty.size);
            }
        }
    }

    for (name, func) in &ctx.functions {
        unsafe { CURRENTFN = name.clone() };
        let mut stack_size = 16;
        // arm64のアラインメントのポリシーはまだ未確認
        for scope in &func.exited_scope_variables {
            for var in scope {
                let mut var = var.borrow_mut();
                var.offset = stack_size;
                stack_size += var.ty.size as isize;
            }
        }
        stack_size = align16(stack_size);

        println!(".text");
        println!(".global _{}", name); // 関数はアンダースコアをつけるのが慣例
        println!("_{}:", name);
        println!("      sub sp, sp, {}", stack_size);
        println!("      stp x29, x30, [sp]");
        println!("      mov x29, sp");

        // 引数の処理
        for (i, arg) in func.args.iter().enumerate() {
            // 他のアドレスを計算する際にx0を使うので、最初の引数のみ特別扱いして対比する
            if i == 0 {
                println!("      mov x9, x0");
                gen_addr(arg.clone());
                println!("      str x9, [x0]");
                continue;
            }
            gen_addr(arg.clone());
            println!("      str x{}, [x0]", i);
        }

        if let Some(body) = &func.body {
            gen_stmt(body.clone());
        }

        println!("end.{}:", name);
        println!("      ldp x29, x30, [sp]");
        println!("      add sp, sp, {}", stack_size);
        println!("      ret");
    }
}
