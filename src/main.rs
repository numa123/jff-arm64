use core::panic;
mod tokenize;
mod types;
use types::Ctx;
mod codegen;
mod parse;
use codegen::*;
mod type_utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("{}: invalid number of arguments", args[0]);
    }
    let input = read_file(args[1].clone().as_str());
    let mut ctx = Ctx {
        input: &input.as_str(),
        input_copy: &input.as_str(),
        tokens: Vec::new(),
        global_variables: Vec::new(),
        processing_funcname: "".to_string(),
        processing_filename: args[1].clone(),
        is_processing_local: false,
        functions: std::collections::HashMap::new(),
    };
    ctx.parse();
    codegen(ctx);
}

fn read_file(file_path: &str) -> String {
    match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => file_path.to_string(),
    }
}
