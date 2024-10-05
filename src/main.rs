struct Ctx<'a> {
    input: &'a str,
    _input_copy: &'a str,
    tokens: Vec<Token>,
}

impl<'a> Ctx<'a> {
    // 入力: 数字から始まる文字列　出力: 数字列。副作用: 文字列を数値の次の文字列まで進める
    fn parse_and_skip_number(&mut self) -> isize {
        let num: String = self.input.chars().take_while(|c| c.is_digit(10)).collect();
        self.input = &self.input[num.len()..];
        return num.parse().unwrap();
    }
}
impl Ctx<'_> {
    fn advance(&mut self, n: usize) {
        self.input = &self.input[n..];
    }
}

#[derive(Debug)]
enum TokenKind {
    TkPunct { str: String },
    TkNum { val: isize },
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
}

impl Ctx<'_> {
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.input.is_empty() {
            let c = self.input.chars().next().unwrap();

            if c == ' ' {
                self.advance(1);
                continue;
            }
            if c.is_digit(10) {
                let num = self.parse_and_skip_number();
                tokens.push(Token {
                    kind: TokenKind::TkNum { val: num },
                });
                continue;
            }
            if c == '+' || c == '-' {
                tokens.push(Token {
                    kind: TokenKind::TkPunct { str: c.to_string() },
                });
                self.advance(1);
                continue;
            }
        }
        return tokens;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("{}: invalid number of arguments", args[0]);
    }
    let input = args[1].clone();

    let mut ctx = Ctx {
        input: &input.as_str(),
        _input_copy: &input.as_str(),
        tokens: Vec::new(),
    };

    ctx.tokens = ctx.tokenize();

    println!(".global _main");
    println!("_main:");

    match ctx.tokens.remove(0).kind {
        TokenKind::TkNum { val } => {
            println!("    mov x0, {}", val);
        }
        _ => panic!("invalid input: {}", ctx.input[0..1].to_string()),
    }

    while !ctx.tokens.is_empty() {
        if let TokenKind::TkPunct { str } = &ctx.tokens[0].kind {
            if str == "+" {
                ctx.tokens.remove(0);
                match ctx.tokens.remove(0).kind {
                    TokenKind::TkNum { val } => {
                        println!("    add x0, x0, {}", val);
                    }
                    _ => panic!("invalid input: {}", ctx.input[0..1].to_string()),
                }
                continue;
            }
            if str == "-" {
                ctx.tokens.remove(0);
                match ctx.tokens.remove(0).kind {
                    TokenKind::TkNum { val } => {
                        println!("    sub x0, x0, {}", val);
                    }
                    _ => panic!("invalid input: {}", ctx.input[0..1].to_string()),
                }
                continue;
            }
        }
        panic!("invalid input: {}", ctx.input[0..1].to_string());
    }
    println!("    ret");
}
