mod codegen;
use codegen::codegen;
mod parse;
use parse::{convert_keywords, parse, tokenize};
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
    convert_keywords(&mut tokens);

    let mut node = parse(&mut tokens, input_copy);

    codegen(&mut node);
}
