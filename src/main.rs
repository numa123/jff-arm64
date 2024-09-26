fn parse_number(p: &mut &str) -> String {
    let num: String = p.chars().take_while(|c| c.is_digit(10)).collect();
    *p = &p[num.len()..];
    return num;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        std::process::exit(1);
    }

    // 数字の後に+が来るか確認する。まずは一桁+一桁のみ
    let mut input = &args[1][..];

    // 最初の数値をx0に入れる
    println!(".global _main");
    println!("_main:");
    println!("  mov x0, {}", parse_number(&mut input));

    while !input.is_empty() {
        if input[0..1].eq("+") {
            input = &input[1..];
            println!("  add x0, x0, {}", parse_number(&mut input));
            continue;
        }
        if input[0..1].eq("-") {
            input = &input[1..];
            println!("  sub x0, x0, {}", parse_number(&mut input));
            continue;
        }
        eprintln!("invalid input: {}", input[0..1].to_string());
        std::process::exit(1);
    }

    println!("  ret");
}
