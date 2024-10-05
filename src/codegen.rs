use crate::types::*;
fn push16() {
    println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき
}
fn pop16() {
    println!("      ldr x1, [sp], 16");
}

fn gen_expr(node: Node) {
    if let NodeKind::NdNum { val } = node.kind {
        println!("      mov x0, {}", val);
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
}

fn gen_stmt(node: Node) {
    if let NodeKind::NdExprStmt { lhs } = node.kind {
        gen_expr(*lhs);
        return;
    }
}

pub fn codegen(program: Vec<Node>) {
    println!(".global _main");
    println!("_main:");

    for stmt in program {
        gen_stmt(stmt);
    }
    println!("      ret");
}
