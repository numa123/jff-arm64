mod codegen;
use codegen::gen_expr;
mod parse;
use parse::{parse, tokenize};
mod types;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        std::process::exit(1);
    }

    let mut input = &args[1][..];
    let input_copy = input; // デバッグ用。とても美しくない

    let mut tokens = tokenize(&mut input);
    let mut node = parse(&mut tokens, input_copy);

    println!(".global _main");
    println!("_main:");
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
    println!("  ret");
}
