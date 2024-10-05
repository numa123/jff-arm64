use core::panic;

struct Ctx<'a> {
    input: &'a str,
    input_copy: &'a str,
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
    fn advance_input(&mut self, n: usize) {
        self.input = &self.input[n..];
    }
    fn current_input_position(&self) -> usize {
        self.input_copy.len() - self.input.len()
    }
}

// for token
impl Ctx<'_> {
    fn get_and_skip_number(&mut self) -> isize {
        match self.tokens[0].kind {
            TokenKind::TkNum { val } => {
                self.tokens.remove(0);
                return val;
            }
            _ => {
                self.error_tok(&self.tokens[0], "expected a number");
                panic!();
            }
        }
    }

    fn advance_tok(&mut self, n: usize) {
        for _ in 0..n {
            self.tokens.remove(0);
        }
    }

    fn error_tok(&self, tok: &Token, msg: &str) {
        eprintln!("{}", self.input_copy);
        eprintln!("{}{}", " ".repeat(tok.start), "^".repeat(tok.len)); // 後々該当箇所のinput_copyを色付けして表す
        eprintln!("jff_error: {}", msg);
        panic!();
    }

    fn error_input_at(&self, msg: &str) {
        eprintln!("{}", self.input_copy);
        eprintln!("{}^", " ".repeat(self.current_input_position()));
        eprintln!("jff_error: {}", msg);
        panic!();
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
    start: usize,
    len: usize,
}

#[derive(Debug)]
enum NodeKind {
    NdAdd { lhs: Box<Node>, rhs: Box<Node> },
    NdSub { lhs: Box<Node>, rhs: Box<Node> },
    NdNum { val: isize },
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
}

fn new_num(val: isize) -> Node {
    Node {
        kind: NodeKind::NdNum { val: val },
    }
}

impl Ctx<'_> {
    fn expr(&mut self) -> Node {
        let mut node = self.primary();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "+" => {
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdAdd {
                            lhs: Box::new(node),
                            rhs: Box::new(self.primary()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdSub {
                            lhs: Box::new(node),
                            rhs: Box::new(self.primary()),
                        },
                    };
                }
                _ => break,
            }
        }
        node
    }

    fn primary(&mut self) -> Node {
        if let TokenKind::TkNum { .. } = self.tokens[0].kind {
            return new_num(self.get_and_skip_number());
        }
        self.error_tok(&self.tokens[0], "expected a number");
        panic!();
    }
}

fn gen_expr(node: Node) {
    if let NodeKind::NdNum { val } = node.kind {
        println!("      mov x0, {}", val);
        return;
    }

    if let NodeKind::NdAdd { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        println!("      str x0, [sp, -16]!  // push"); // 16はハードコードだが、スタックのサイズを計算して動的にするべき
        gen_expr(*rhs);
        println!("      ldr x1, [sp], 16 // pop"); // lhsの計算結果がx1, rhsの計算結果がx0に入る
        println!("      add x0, x1, x0");
        return;
    }

    if let NodeKind::NdSub { lhs, rhs } = node.kind {
        gen_expr(*lhs);
        println!("      str x0, [sp, -16]!  // push");
        gen_expr(*rhs);
        println!("      ldr x1, [sp], 16 // pop");
        println!("      sub x0, x1, x0");
        return;
    }
}

// start, endの計算がわかりづらい
impl Ctx<'_> {
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.input.is_empty() {
            let c = self.input.chars().next().unwrap();

            if c == ' ' {
                self.advance_input(1);
                continue;
            }
            if c.is_digit(10) {
                let num = self.parse_and_skip_number();
                tokens.push(Token {
                    kind: TokenKind::TkNum { val: num },
                    start: self.current_input_position() - num.to_string().len(),
                    len: num.to_string().len(),
                });
                continue;
            }
            if c == '+' || c == '-' {
                tokens.push(Token {
                    kind: TokenKind::TkPunct { str: c.to_string() },
                    start: self.current_input_position(),
                    len: 1,
                });
                self.advance_input(1);
                continue;
            }
            self.error_input_at(
                format!("invalid input: {}", self.input[0..1].to_string()).as_str(),
            );
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
        input_copy: &input.as_str(),
        tokens: Vec::new(),
    };

    ctx.tokens = ctx.tokenize();

    println!(".global _main");
    println!("_main:");

    let node = ctx.expr();
    // eprintln!("{:#?}", node);
    gen_expr(node);

    // println!("    mov x0, {}", ctx.get_and_skip_number());

    // while !ctx.tokens.is_empty() {
    //     if let TokenKind::TkPunct { str } = &ctx.tokens[0].kind {
    //         if str == "+" {
    //             ctx.tokens.remove(0);
    //             println!("    add x0, x0, {}", ctx.get_and_skip_number());
    //             continue;
    //         }
    //         if str == "-" {
    //             ctx.tokens.remove(0);
    //             println!("    sub x0, x0, {}", ctx.get_and_skip_number());
    //             continue;
    //         }
    //     }
    //     ctx.error_tok(&ctx.tokens[0], "invalid token");
    // }
    println!("      ret");
}
