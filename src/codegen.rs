use crate::types::{Function, Node, NodeKind};

pub static mut BCOUNT: usize = 0; // branch count

// 左辺値のアドレスをx0に入れる処理
fn gen_addr(node: Node) {
    match node.kind {
        NodeKind::NdVar => {
            let offset = (node.var.unwrap().offset + 2) * 8; // 例えば、str x0, [x29, -16]とすると、x29-16 ~ x29-24ではなく、x29-16 ~ x29-8になる。
                                                             // offset設定の際、現在は0から設定しているので、+1しないと、最初の変数がx29~x29+8になってしまって、lp(x30)と被ってしまう(と解釈している)
                                                             // "+2"は、fp(x29)とlr(x30)の分を考慮している
            println!("  add x0, x29, {}", offset);
            return;
        }
        NodeKind::NdDeref => {
            gen_expr(*(node.lhs).unwrap());
            return;
        }
        _ => {
            eprintln!("not a lvalue");
            panic!();
        }
    }
}

// x0に値を入れる処理
pub fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::NdNum => {
            println!("  mov x0, {}", node.val);
            return;
        }
        NodeKind::NdNeg => {
            gen_expr(*(node.lhs).unwrap()); // lhsはあるはずだから、panicしても良いためunwrap
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
            println!("  str x0, [sp, -16]!"); // push  16で果たして良いのかは不明。ただ、ここでストアした値は後々消えるので、今は問題ない。
                                              // ただ、扱う値が16バイトを超える(16バイトでも起こるかもしれないが)場合は、もっと大きくしないといけないと予想している
            gen_expr(*(node.rhs).unwrap()); // rhsはx0に入るはずだから、panicしても良いためunwrap
            println!("  ldr x1, [sp], 16"); // pop to x1 x1には左辺値のアドレスが入っているはず。
                                            // ldrのポストインデックス。spから読み出した値をx0に格納し、spを16増やす
            println!("  str x0, [x1]"); // a=1;の場合、aのアドレスに1を入れる処理
            return;
        }
        NodeKind::NdAddr => {
            gen_addr(*(node.lhs).unwrap());
            return;
        }
        NodeKind::NdDeref => {
            gen_expr(*(node.lhs).unwrap());
            println!("  ldr x0, [x0]");
            return;
        }
        _ => {}
    }

    if let Some(lhs) = node.lhs {
        gen_expr(*lhs);
        println!("  str x0, [sp, -16]!"); // push  上と同様
    }
    if let Some(rhs) = node.rhs {
        gen_expr(*rhs);
        println!("  ldr x1, [sp], 16"); // pop to x1
                                        // 今、rhsの計算結果がx0, lhsの計算結果がx1に入っている
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
            println!("  sdiv x0, x1, x0"); // sdivで0徐算をすると、0を返すようになっているっぽい
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
            for n in &node.args {
                gen_expr(n.clone());
                println!("  str x0, [sp, -16]!",); // 引数はスタックに積む。
                                                   // またもや16バイトであり、16バイトより大きいデータ型を扱う必要が出てきたら、変えなければならない。具体的な状況は今は思いついていないけど。
            }
            for i in (0..node.args.len()).rev() {
                println!("  ldr x{i}, [sp], 16");
            }

            println!("  bl _{}", node.func_name)
        }
        _ => {
            eprintln!("invalid node kind");
            panic!();
        }
    }
}

// 文の生成
fn gen_stmt(node: Node, func_name: &str) {
    match node.kind {
        NodeKind::NdExprStmt => {
            gen_expr(*(node.lhs).unwrap());
        }
        NodeKind::NdReturn => {
            gen_expr(*(node.lhs).unwrap());
            println!("  b end{}", func_name);
        }
        NodeKind::NdBlock => {
            for node in node.block_body {
                gen_stmt(node, func_name);
            }
        }
        NodeKind::NdIf => {
            let count = unsafe { BCOUNT };
            gen_expr(*(node.cond).unwrap()); // x0に条件式の結果が入る。x0が1ならthne, 0ならelsを実行するようにジャンプ命令を生成する。この時ジャンプ先命令が一意になるように識別子をつけないといけない。また構造体に格納するモチベーションが生まれた
            println!("  cmp x0, 1");
            println!("  b.eq then{}", count);
            println!("  b.ne else{}", count);
            println!("then{}:", count);
            gen_stmt(*(node.then).unwrap(), func_name);
            println!("  b end{}", count);
            println!("else{}:", count);
            if let Some(els) = node.els {
                gen_stmt(*els, func_name);
                println!("  b end{}", count);
            }
            println!("end{}:", count);
            unsafe { BCOUNT += 1 };
        }
        NodeKind::NdFor => {
            let count = unsafe { BCOUNT };
            if node.init.is_some() {
                gen_stmt(*(node.init).unwrap(), func_name); // 間違えてgen_exprの時があった。
            }
            // 2回もnode.condがあるかどうかを確かめているのは良くない気がするけど、2回だしまあよしとしている部分
            if node.cond.is_some() {
                println!("  b check_cond{}", count);
            }
            println!("start{}:", count);
            gen_stmt(*(node.then).unwrap(), func_name);
            if let Some(inc) = node.inc {
                gen_expr(*inc);
            }
            if let Some(cond) = node.cond {
                println!("check_cond{}:", count);
                gen_expr(*cond);
                println!("  cmp x0, 1");
                println!("  b.eq start{}", count);
                println!("  b.ne end{}", count);
            }
            println!("end{}:", count);
        }
        _ => {
            eprintln!("invalid node kind");
            panic!();
        }
    }
}

fn align_16(n: usize) -> usize {
    (n + 15) & !15
}

fn gen_args_prologue(args: &Vec<Node>) {
    for (i, arg) in args.iter().enumerate() {
        // 他のアドレスを計算する際、x0を使うので、最初の引数のみ特別扱いして退避する
        if i == 0 {
            println!("  mov x9, x0");
            gen_addr(arg.clone()); // x0にアドレスが入る
            println!("  str x9, [x0]");
            continue;
        }
        gen_addr(arg.clone()); // x0にアドレスが入る
        println!("  str x{}, [x0]", i);
    }
}

pub fn codegen(funcs: &mut Vec<Function>) {
    for f in funcs {
        let stack_size = f.variables.len() * 8; // デバッグなど用のwzr, lp, fpは含めない、ローカル変数のみのスタックサイズ
                                                // 今はlongのみのサポートを想定しているから8バイトずつ確保している(つもり)
        let prorogue_size = align_16(stack_size) + 16;
        println!(".global _main");
        println!("_{}:", f.name);
        // プロローグ
        // 無駄が多くなるが動くのでよしとしている。関数呼び出しの有無、変数宣言の有無などによって変化する。
        // subがなかったり、sturがなかったり、mov x29, spになっていたり。
        println!("  stp x29, x30, [sp, -{}]!", prorogue_size);
        println!("  mov x29, sp");

        // 関数の引数をレジスターに詰めていく。普通の変数を設定するのと同じように
        gen_args_prologue(&f.args);

        while !f.stmts.is_empty() {
            gen_stmt(f.stmts[0].clone(), &f.name); // こうしないとnodeの所有権が移動してしまう。gen_exprを変えれば良いが一旦これで。
            f.stmts.remove(0);
        }

        println!("end{}:", f.name);
        println!("  ldp x29, x30, [sp] ,{}", prorogue_size);
        println!("  ret");
    }
}
