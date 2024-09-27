use crate::types::{Node, NodeKind};

fn gen_addr(node: Node) {
    if node.kind == NodeKind::NdVar {
        // println!("{:?}", node); デバッグ用
        let offset = node.var.unwrap().offset * 8; // sp + 16 + offsetでアドレスを計算
        println!("  add x0, x29, {}", offset);
        return;
    }
    eprintln!("not a lvalue");
}

pub fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::NdNum => {
            println!("  mov x0, {}", node.val);
            return;
        }
        NodeKind::NdNeg => {
            gen_expr(*(node.lhs).unwrap()); // lhsはあるはず
            println!("  neg x0, x0");
            return;
        }
        NodeKind::NdVar => {
            gen_addr(node);
            println!("  ldr x0, [x0]");
            return;
        }
        NodeKind::NdAssign => {
            gen_addr(*(node.lhs).unwrap());
            println!("  str x0, [sp, -16]!"); // push  16で果たして良いのか、でも8じゃ動かなかった気がするからなあ。いやこれlp, fpのあれか
            gen_expr(*(node.rhs).unwrap()); // rhsはx0に入るはず
            println!("  ldr x1, [sp], 16"); // pop to x1 x1には左辺値のアドレスが入っているはず
            println!("  str x0, [x1]"); // a=1;なら、aのアドレスに1を入れる
            return;
        }
        _ => {}
    }

    // node.lhs, node.rhsがあれば、それを再帰的に呼び出す
    if let Some(lhs) = node.lhs {
        gen_expr(*lhs);
        println!("  str x0, [sp, -16]!"); // push  16バイトアラインメントしているのか？。変数をアリにしたらここも変えないといけない気がするな
    }
    if let Some(rhs) = node.rhs {
        gen_expr(*rhs);
    }
    println!("  ldr x1, [sp], 16"); // pop to x1
                                    // 今、rhsの計算結果がx0, lhsの計算結果がx1に入っている
                                    // 変数をアリにしたらここも変えないといけない気がするな

    match node.kind {
        NodeKind::NdAdd => {
            println!("  add x0, x1, x0");
        }
        NodeKind::NdSub => {
            println!("  sub x0, x1, x0");
        }
        NodeKind::NdMul => {
            println!("  mul x0, x1, x0");
        }
        NodeKind::NdDiv => {
            // x0が0だった場合は未定義になりそう？そのためにはcmpとか？一旦後で
            println!("  cbz x0, error"); // x0が0の場合はエラー処理に飛ぶ
            println!("  sdiv x0, x1, x0");
        }
        NodeKind::NdEq => {
            println!("  cmp x1, x0");
            println!("  cset x0, eq");
        }
        NodeKind::NdNe => {
            println!("  cmp x1, x0");
            println!("  cset x0, ne");
        }
        NodeKind::NdLt => {
            println!("  cmp x1, x0");
            println!("  cset x0, lt");
        }
        NodeKind::NdLe => {
            println!("  cmp x1, x0");
            println!("  cset x0, le");
        }
        NodeKind::NdGt => {
            println!("  cmp x1, x0");
            println!("  cset x0, gt");
        }
        NodeKind::NdGe => {
            println!("  cmp x1, x0");
            println!("  cset x0, ge");
        }
        _ => eprintln!("invalid node kind"),
    }
}

pub fn codegen(node: &mut Vec<Node>) {
    println!(".global _main");
    println!("_main:");
    // プロローグ
    println!("  sub sp, sp, 240");
    // println!("  stp x29, x30, [sp, #16]"); // これは関数内で関数を呼び出すときだけ必要なのかもしれない。
    // println!("  add x29, sp, #16");

    while !node.is_empty() {
        gen_expr(node[0].clone()); // こうしないとnodeの所有権が移動してしまう。gen_exprを変えれば良いが一旦これで。
        node.remove(0);
    }
    println!("  b end");

    // ゼロ徐算の場合のエラー処理
    println!("error:");
    println!("  mov x0, 1");
    println!("  b end"); // これじゃあただ正常に計算結果が1なのか、エラーが1なのかわからないのでは？まあ今は動くのでよしとする

    println!("end:");
    // println!("  ldp x29, x30, [sp, #16]"); // これは関数内で関数を呼び出すときだけ必要なのかもしれない。
    println!("  add sp, sp, #240");
    println!("  ret");
}
