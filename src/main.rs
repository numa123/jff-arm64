#[derive(Debug)]
enum TokenKind {
    TkPunct,
    TkNum,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    val: i32,
    str: String,
    loc: usize,
}

fn tokenize(p: &mut &str) -> Vec<Token> {
    let p_copy = *p;
    let mut tokens = Vec::new();
    let mut index = 0;
    while !p.is_empty() {
        let c = p.chars().next().unwrap();
        if c == ' ' {
            *p = &p[1..];
            index += 1;
            continue;
        }
        if c.is_digit(10) {
            let num = parse_number(p);
            tokens.push(Token {
                kind: TokenKind::TkNum,
                val: num.parse().unwrap(),
                str: num.clone(),
                loc: index,
            });
            index += num.len();
            continue;
        }
        if c == '+' || c == '-' {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: p[0..1].to_string(),
                loc: index,
            });
            *p = &p[1..];
            index += 1;
            continue;
        }
        eprintln!("{}", p_copy);
        eprintln!("{}^", " ".repeat(index));
        eprintln!("invalid token");
        std::process::exit(1);
    }
    return tokens;
}

fn parse_number(p: &mut &str) -> String {
    let num: String = p.chars().take_while(|c| c.is_digit(10)).collect();
    *p = &p[num.len()..]; // これは関数の外に出した方が明示的に書きやすいかも？
    return num;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}: invalid number of arguments", args[0]);
        std::process::exit(1);
    }

    let mut input = &args[1][..];
    let input_copy = input; // デバッグ用。とても美しくない
    let mut tokens = tokenize(&mut input);
    // println!("{:?}", tokens);

    // 最初の文字は数値
    println!(".global _main");
    println!("_main:");
    println!("  mov x0, {}", get_number(&tokens[0], input_copy));
    tokens.remove(0); // 1つ配列を消費

    while !tokens.is_empty() {
        if tokens[0].str.eq("+") {
            tokens.remove(0);
            println!("  add x0, x0, {}", get_number(&tokens[0], input_copy));
            tokens.remove(0);
            continue;
        }
        if tokens[0].str.eq("-") {
            tokens.remove(0);
            println!("  sub x0, x0, {}", get_number(&tokens[0], input_copy));
            tokens.remove(0);
            continue;
        }
        eprintln!("invalid input: {}", get_number(&tokens[0], input_copy));
        std::process::exit(1);
    }

    println!("  ret");
}

fn get_number(t: &Token, input: &str) -> i32 {
    match t.kind {
        TokenKind::TkNum => t.val,
        _ => {
            eprintln!("{}", input);
            eprintln!("{}^", " ".repeat(t.loc));
            eprintln!("expected a number");
            std::process::exit(1);
        }
    }
}
