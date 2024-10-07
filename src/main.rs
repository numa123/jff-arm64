use core::panic;
mod tokenize;
mod types;
use types::Ctx;
mod codegen;
mod parse;
use codegen::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("{}: invalid number of arguments", args[0]);
    }
    let input = args[1].clone();
    let mut ctx = Ctx {
        input: &input.as_str(),
        input_copy: &input.as_str(),
        tokens: Vec::new(),
        global_variables: Vec::new(),
        // global_stmts: Vec::new(),
        processing_funcname: "".to_string(), // defualt
        is_processing_local: false,
        functions: std::collections::HashMap::new(),
    };
    ctx.parse();
    codegen(ctx);
}
