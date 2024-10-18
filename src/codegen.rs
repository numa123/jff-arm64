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
        TypeKind::Array { .. } | TypeKind::Struct { .. } | TypeKind::Union { .. } => {
            return;
        }
        _ => {}
    }
    match ty.size {
        1 => {
            println!("      ldrsb x0, [x0]");
        }
        2 => {
            println!("      ldrsh w0, [x0]")
        }
        4 => {
            println!("      ldr w0, [x0]");
        }
        _ => {
            println!("      ldr x0, [x0]");
        }
    }
}

fn store(ty: &Type) {
    if let TypeKind::Struct { .. } | TypeKind::Union { .. } = &ty.kind {
        for i in 0..ty.size {
            println!("      ldrb w2, [x0, {}]", i);
            println!("      strb w2, [x1, {}]", i);
        }
        return;
    }
    match ty.size {
        1 => {
            println!("      strb w0, [x1]");
        }
        2 => {
            println!("      strh w0, [x1]")
        }
        4 => {
            println!("      str w0, [x1]");
        }
        _ => {
            println!("      str x0, [x1]");
        }
    }
}

// 多分ガバガバ
fn cast(from: Type, to: Type) {
    let instruction = match (from.clone().kind, to.clone().kind) {
        (TypeKind::Char, TypeKind::Short) => "      sxtb w0, w0",
        (TypeKind::Char, TypeKind::Int) => "      sxtb w0, w0",
        (TypeKind::Char, TypeKind::Long) => "      sxtb x0, w0",
        (TypeKind::Char, TypeKind::Char) => "",

        (TypeKind::Short, TypeKind::Char) => "      sxtb w0, w0",
        (TypeKind::Short, TypeKind::Int) => "      sxth w0, w0",
        (TypeKind::Short, TypeKind::Long) => "      sxth x0, w0",
        (TypeKind::Short, TypeKind::Short) => "",

        (TypeKind::Int, TypeKind::Char) => "      sxtb w0, w0",
        (TypeKind::Int, TypeKind::Short) => "      sxth w0, w0",
        (TypeKind::Int, TypeKind::Long) => "      sxtw x0, w0",
        (TypeKind::Int, TypeKind::Int) => "",

        (TypeKind::Long, TypeKind::Char) => "      sxtb w0, w0",
        (TypeKind::Long, TypeKind::Short) => "      sxth w0, w0",
        (TypeKind::Long, TypeKind::Int) => "      sxtw x0, w0",
        (TypeKind::Long, TypeKind::Long) => "",

        _ => {
            eprintln!("from type: {:#?}", from);
            eprintln!("to type: {:#?}", to);
            panic!("not supported cast type")
        }
    };
    println!("{}", instruction);
}

fn gen_addr(node: Node) {
    match node.kind {
        NodeKind::Var { var } => {
            let var = var.borrow();
            if var.is_local {
                println!("      add x0, x29, {}", var.offset);
            } else {
                println!("      adrp x0, {}@PAGE", var.name); // what is PAGE?
                println!("      add x0, x0, {}@PAGEOFF;", var.name);
            }
        }
        NodeKind::Deref { lhs, .. } => {
            gen_expr(*lhs);
        }
        NodeKind::Member { lhs, member } => {
            gen_addr(*lhs);
            println!("      add x0, x0, {}", member.offset);
        }
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::Num { val } => {
            println!("      mov x0, {}", val);
        }
        NodeKind::Var { ref var } => {
            let ty = var.borrow().ty.clone();
            gen_addr(node);
            load(&ty);
        }
        NodeKind::Cas { lhs } => {
            let to_ty = node.ty.clone().unwrap();
            let from_ty = lhs.ty.clone().unwrap();
            gen_expr(*lhs);
            cast(from_ty, to_ty);
        }
        NodeKind::Member { .. } => {
            let ty = node.clone().ty.unwrap();
            gen_addr(node); // x.valとかだったら、xのアドレスをx0に入れる。
            load(&ty);
        }
        NodeKind::Add { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      add x0, x1, x0");
        }
        NodeKind::Sub { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sub x0, x1, x0");
        }
        NodeKind::Mul { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      mul x0, x1, x0");
        }
        NodeKind::Div { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sdiv x0, x1, x0");
        }
        NodeKind::Mod { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      sdiv x2, x1, x0");
            println!("      msub x0, x2, x0, x1");
        }
        NodeKind::Neg { lhs } => {
            gen_expr(*lhs);
            println!("      neg x0, x0");
        }
        NodeKind::Eq { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x0, x1");
            println!("      cset x0, eq");
        }
        NodeKind::Ne { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x0, x1");
            println!("      cset x0, ne");
        }
        NodeKind::Lt { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, lt");
        }
        NodeKind::Le { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, le");
        }
        NodeKind::Gt { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, gt");
        }
        NodeKind::Ge { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      cmp x1, x0");
            println!("      cset x0, ge");
        }
        NodeKind::And { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      mov x2, 0");
            println!("      cmp x1, 0");
            println!("      cset x2, ne");
            println!("      cmp x0, 0");
            println!("      cset x0, ne");
            println!("      and x0, x0, x2");
        }
        NodeKind::Or { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      mov x2, 0");
            println!("      cmp x1, 0");
            println!("      cset x2, ne");
            println!("      cmp x0, 0");
            println!("      cset x0, ne");
            println!("      orr x0, x0, x2");
        }
        NodeKind::BitAnd { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      and x0, x1, x0");
        }
        NodeKind::BitXor { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      eor x0, x1, x0");
        }
        NodeKind::BitOr { lhs, rhs } => {
            gen_expr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            println!("      orr x0, x1, x0");
        }
        NodeKind::NdAssign { lhs, rhs } => {
            gen_addr(*lhs);
            push16();
            gen_expr(*rhs);
            pop16();
            store(node.ty.as_ref().unwrap()); // unwrap使わずにいけないかな
        }
        NodeKind::Addr { lhs } => {
            gen_addr(*lhs);
        }
        NodeKind::Deref { lhs, .. } => {
            gen_expr(*lhs);
            load(node.ty.as_ref().unwrap()); // 正しいか？
        }
        NodeKind::FuncCall { name, args } => {
            for arg in &args {
                gen_expr(arg.clone());
                push16();
            }
            for i in (0..args.len()).rev() {
                println!("      ldr x{}, [sp], 16 // pop for function arg", i);
            }
            println!("      bl _{}", name);
        }
        NodeKind::GNUStmtExpr { body } => {
            for stmt in body {
                gen_stmt(stmt);
            }
        }
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn gen_stmt(node: Node) {
    // eprintln!("gen_stmt: {:#?}", node);
    match node.kind {
        NodeKind::ExprStmt { lhs } => {
            gen_expr(*lhs);
        }
        NodeKind::Return { lhs } => {
            gen_expr(*lhs);
            println!("      b end.{}", unsafe { CURRENTFN.clone() });
        }
        NodeKind::Block { body } => {
            for stmt in body {
                gen_stmt(stmt);
            }
        }
        NodeKind::If { cond, then, els } => {
            let idx = unsafe { IFIDX };
            unsafe { IFIDX += 1 };
            gen_expr(*cond);
            println!("      cmp x0, 1");
            if let Some(els) = els {
                println!("      b.ne else.{}", idx);
                gen_stmt(*then);
                println!("      b endif.{}", idx);
                println!("else.{}:", idx);
                gen_stmt(*els);
            } else {
                println!("      b.ne endif.{}", idx);
                gen_stmt(*then);
            }
            println!("endif.{}:", idx);
        }
        NodeKind::For {
            init,
            cond,
            inc,
            body,
        } => {
            let idx = unsafe { FORIDX };
            unsafe { FORIDX += 1 };
            gen_stmt(*init);
            println!("      b cond.{}", idx);
            println!("startfor.{}:", idx);
            gen_stmt(*body);
            if let Some(inc) = inc {
                gen_expr(*inc);
            }
            println!("cond.{}:", idx);
            if let Some(cond) = cond {
                gen_expr(*cond);
                println!("      cmp x0, 1");
                println!("      b.ne endfor.{}", idx);
            }
            println!("      b startfor.{}", idx);
            println!("endfor.{}:", idx);
        }
        NodeKind::While { cond, body } => {
            let idx = unsafe { FORIDX };
            unsafe { FORIDX += 1 };
            println!("startwhile.{}:", idx);
            gen_expr(*cond);
            println!("      cmp x0, 1");
            println!("      b.ne endwhile.{}", idx);
            gen_stmt(*body);
            println!("      b startwhile.{}", idx);
            println!("endwhile.{}:", idx);
        }
        _ => panic!("not expected node: {:#?}", node),
    }
}

fn align_to(n: usize, to: usize) -> usize {
    if to == 0 {
        return n; // なぜreturnを書く必要がある？ nではだめなのか
    }
    (n + to - 1) & !(to - 1)
}

// 関数以外のグローバル変数
fn handle_data(ctx: &Ctx) {
    for var in &ctx.gvars {
        let var = var.borrow();

        // 初期値がない場合の処理
        if var.init_gval.is_none() {
            println!(".data");
            println!(".global {}", var.name);
            println!("{}:", var.name);
            println!("      .zero {}", var.ty.size);
            continue;
        }

        // 初期値がある場合の処理
        match &var.init_gval.as_ref().unwrap() {
            InitGval::Str(s) => {
                let trimmed = s.trim_end_matches('\0'); // ヌル文字を除去
                println!(".text");
                println!(".cstring"); // セクションの指定
                println!(".align 3"); // ポインタは8byte。align 3 は　2^3 = 8byteでアラインメント
                println!("{}:", var.name);
                if trimmed.is_empty() {
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
}

// 関数
fn handle_text(ctx: &Ctx) {
    for (name, func) in &ctx.functions {
        // 宣言のみの場合はスキップ
        if !func.is_def {
            continue;
        }
        unsafe { CURRENTFN = name.clone() };
        let mut stack_size = 16; // fp, lp用に事前確保
        for scope in &func.exited_scope {
            for var in &scope.variables {
                let mut var = var.borrow_mut();
                stack_size = align_to(stack_size, var.ty.align);
                var.offset = stack_size;
                stack_size += var.ty.size; // もしかしたら撮りすぎかも。alignをうまく使う？
                                           // eprintln!("var:{:#?}", var);
            }
        }
        stack_size = align_to(stack_size, 16);

        println!(".text");
        println!(".align 2");
        println!(".global _{}", name); // 関数はアンダースコアをつけるのが慣例
        println!("_{}:", name);
        println!("      sub sp, sp, {}", stack_size);
        println!("      stp x29, x30, [sp]");
        println!("      mov x29, sp");

        // 引数の処理
        // chibiccだと、関数の引数でもレジスタの選別をしていた。
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

pub fn codegen(ctx: Ctx) {
    handle_data(&ctx);
    handle_text(&ctx);
}
