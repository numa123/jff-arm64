fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        std::process::exit(1);
    }
    println!(".global _main");
    println!("_main:");
    println!("    mov x0, #{}", args[1]);
    println!("    ret");
}
