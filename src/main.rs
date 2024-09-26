#[derive(Debug)]
enum TokenKind {
    TK_PUNCT,
    TK_NUM,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    val: i32,
    str: String,
}

fn tokenize(p: &mut &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    while !p.is_empty() {
        let c = p.chars().next().unwrap();

        if c == ' ' {
            *p = &p[1..];
            continue;
        }
        if c.is_digit(10) {
            let num = parse_number(p);
            tokens.push(Token {
                kind: TokenKind::TK_NUM,
                val: num.parse().unwrap(),
                str: num,
            });
            continue;
        }
        if c == '+' || c == '-' {
            tokens.push(Token {
                kind: TokenKind::TK_PUNCT,
                val: 0,
                str: p[0..1].to_string(),
            });
            *p = &p[1..];
            continue;
        }
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
    let mut tokens = tokenize(&mut input);

    // 最初の文字は数値
    println!(".global _main");
    println!("_main:");
    println!("  mov x0, {}", tokens[0].val);
    tokens.remove(0); // 1つ配列を消費

    while !tokens.is_empty() {
        if tokens[0].str.eq("+") {
            tokens.remove(0);
            println!("  add x0, x0, {}", tokens[0].val);
            tokens.remove(0);
            continue;
        }
        if tokens[0].str.eq("-") {
            tokens.remove(0);
            println!("  sub x0, x0, {}", tokens[0].val);
            tokens.remove(0);
            continue;
        }
        eprintln!("invalid input: {}", tokens[0].str);
        std::process::exit(1);
    }

    println!("  ret");
}
