struct Ctx<'a> {
    input: &'a str,
    _input_copy: &'a str,
}

impl<'a> Ctx<'a> {
    // 入力: 数字から始まる文字列　出力: 数字列。副作用: 文字列を数値の次の文字列まで進める
    fn parse_and_skip_number(&mut self) -> String {
        let num: String = self.input.chars().take_while(|c| c.is_digit(10)).collect();
        self.input = &self.input[num.len()..];
        return num;
    }
}
impl Ctx<'_> {
    fn advance(&mut self, n: usize) {
        self.input = &self.input[n..];
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
    };

    println!(".global _main");
    println!("_main:");
    println!("    mov x0, {}", ctx.parse_and_skip_number());

    while !ctx.input.is_empty() {
        if ctx.input[0..1].eq("+") {
            ctx.advance(1);
            println!("    add x0, x0, {}", ctx.parse_and_skip_number());
            continue;
        }
        if ctx.input[0..1].eq("-") {
            ctx.advance(1);
            println!("    sub x0, x0, {}", ctx.parse_and_skip_number());
            continue;
        }
        panic!("invalid input: {}", ctx.input[0..1].to_string());
    }
    println!("    ret");
}
