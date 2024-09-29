use crate::parse::{HASFUNCCALL, VARIABLES};
use crate::types::{Node, NodeKind};

pub static mut BCOUNT: usize = 0; // branch count

fn gen_addr(node: Node) {
    if node.kind == NodeKind::NdVar {
        // println!("{:?}", node); デバッグ用
        let offset = (node.var.unwrap().offset + 1) * 8; // sp + 16 + offsetでアドレスを計算 // 多分+1
        println!("  add x0, x29, -{}", offset);
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
            println!("  str x0, [sp, -16]!"); // push  16で果たして良いのか、でも8じゃ動かなかった気がするからなあ。いやこれlp, fpのあれか。これちょっとspをいじるっていう点で危険では？まあ良いのか。
                                              // 今数値をlongにしているのと、spは16バイトアラインメントされる必要がありそうだから(多分)、16でやるのは無駄遣いかもしれないけど、動きはする。
                                              // spを動かす時は、必ず16バイトアラインメントするように変更するなどが必要かもしれない。
                                              // longにしていると思っているのは自分だけかもしれない？
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
        println!("  ldr x1, [sp], 16"); // pop to x1
                                        // 今、rhsの計算結果がx0, lhsの計算結果がx1に入っている
                                        // 変数をアリにしたらここも変えないといけない気がするな
                                        // これ、ifの中にないといけなかった。そうしないとsegmentation faultがおこっちゃう。spがおかしくなってしまうから。だと思う。
    }

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
        NodeKind::NdFuncCall => {
            println!("  bl _{}", node.funcname);
        }
        _ => eprintln!("invalid node kind"),
    }
}

fn gen_stmt(node: Node) {
    match node.kind {
        NodeKind::NdExprStmt => {
            gen_expr(*(node.lhs).unwrap());
        }
        NodeKind::NdReturn => {
            gen_expr(*(node.lhs).unwrap());
            println!("  b end"); // これretは不適切でした！
        }
        NodeKind::NdBlock => {
            for node in node.block_body {
                gen_stmt(node);
            }
        }
        NodeKind::NdIf => {
            let count = unsafe { BCOUNT };
            gen_expr(*(node.cond).unwrap()); // x0に条件式の結果が入る。x0が1ならthne, 0ならelsを実行するようにジャンプ命令を生成する。この時ジャンプ先命令が一意になるように識別子をつけないといけない。また構造体に格納するモチベーションが生まれた
            println!("  cmp x0, 1");
            println!("  b.eq then{}", count);
            println!("  b.ne else{}", count);
            println!("then{}:", count);
            gen_stmt(*(node.then).unwrap());
            println!("  b end{}", count);
            println!("else{}:", count);
            if let Some(els) = node.els {
                gen_stmt(*els);
                println!("  b end{}", count);
            }
            println!("end{}:", count);
            unsafe { BCOUNT += 1 };
        }
        NodeKind::NdFor => {
            // initをやり、condを評価して、中身を実行し、incをやる。initは最初だけか。
            let count = unsafe { BCOUNT };
            if node.init.is_some() {
                gen_stmt(*(node.init).unwrap()); // 間違えてgen_exprの時があった。
            }
            if node.cond.is_some() {
                // 2回もnode.condがあるかどうかを確かめているのは良くない気がする。けどまあ動く。
                println!("  b check_cond{}", count);
            }
            println!("start{}:", count);
            gen_stmt(*(node.then).unwrap());
            if let Some(inc) = node.inc {
                gen_expr(*inc);
            }
            if let Some(cond) = node.cond {
                println!("check_cond{}:", count); // これでちょっと不適切な気がする
                gen_expr(*cond);
                println!("  cmp x0, 1");
                println!("  b.eq start{}", count);
                println!("  b.ne end{}", count);
            }
            println!("end{}:", count);
        }
        _ => eprintln!("invalid node kind"),
    }
}
fn align_16(size: usize) -> usize {
    size / 16 * 16 + 16
}

pub fn codegen(node: &mut Vec<Node>) {
    let stack_size = unsafe { VARIABLES.len() * 8 }; // デバッグなど用のwzr, lp, fpは含めない、ローカル変数のみのスタックサイズ。今はlongのみのサポートだから*8
    let prorogue_size = unsafe {
        if HASFUNCCALL {
            align_16(stack_size + 4) + 16
        } else {
            align_16(stack_size + 4)
        }
    };
    println!(".global _main");
    println!("_main:");
    // プロローグ
    println!("  sub sp, sp, {}", prorogue_size); // 関数を実行するだけのmain関数であればこれすらいらないみたい。でもその場合、stp x29, 30とかは必要っぽい。あとで整理する必要がある。それがABI的に正しければの話だけど。
                                                 // まあいっぱい確保しちゃうっていうだけで、それ以外に影響はないから、問題が発生するまではこれで良いような気持ちもある。
    println!("  stp x29, x30, [sp, #{}]", prorogue_size - 16);
    println!("  add x29, sp, #{}", prorogue_size - 16); // ここ本当は、HASFUNCCALLがtrueかつ、変数宣言があるかどうかっぽい。
    println!("  stur wzr, [x29, #-4]",); // こっちもHASFUNCCALLがtrueかつ、変数宣言があるかどうかっぽくて

    while !node.is_empty() {
        gen_stmt(node[0].clone()); // こうしないとnodeの所有権が移動してしまう。gen_exprを変えれば良いが一旦これで。
        node.remove(0);
    }
    println!("  b end");

    // ゼロ徐算の場合のエラー処理
    println!("error:");
    println!("  mov x0, 1");
    println!("  b end"); // これじゃあただ正常に計算結果が1なのか、エラーが1なのかわからないのでは？まあ今は動くのでよしとする

    println!("end:");
    println!("  ldp x29, x30, [sp, #{}]", prorogue_size - 16); // これは関数内で関数を呼び出すときだけ必要なのかもしれない。
    println!("  add sp, sp, #{}", prorogue_size);
    println!("  ret");
}
