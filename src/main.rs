use core::panic;
use std::collections::HashMap;
mod tokenize;
mod types;
use types::Ctx;
mod codegen;
mod parse;
use codegen::*;
mod new_node;
mod type_utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("{}: invalid number of arguments", args[0]);
    }
    let input = read_file(args[1].clone().as_str());
    let mut ctx = Ctx {
        input: input.as_str(),
        input_copy: input.as_str(),
        tokens: Vec::new(),
        consumed_tokens: Vec::new(),
        gvars: Vec::new(),
        cur_func: "".to_string(),
        cur_file: args[1].clone(),
        functions: HashMap::new(),
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
