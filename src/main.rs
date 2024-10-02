mod codegen;
use codegen::codegen;
mod parse;
use parse::parse;
mod tokenize;
use tokenize::{convert_keywords, tokenize};
mod types;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        panic!();
    }

    let mut input = &args[1][..];
    let input_copy = input; // デバッグ用。他にやりようがありそう -> 構造体のメンバに入れるのがよさそうかな
    let mut tokens = tokenize(&mut input);
    convert_keywords(&mut tokens);
    let mut funcs = parse(&mut tokens, input_copy);
    codegen(&mut funcs);
}

// プログラム全体の構造体
// 入力プログラム
// (トークン列のコピー)
// 処理中のトークン列
// プログラムを構成する関数のリスト
// ブランチカウントBCOUNT

// こんな感じの構造体によって、関数へのtokens, input, vを引数で渡したり、b end用の関数名も渡さずに済むようになる
// ただ、もっと複雑なプログラムをコンパイルできて軌道に乗ってきてからやる方が良いのではないかと考えている
