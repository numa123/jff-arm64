#[derive(Debug, PartialEq)]
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

#[derive(Clone, PartialEq)]
enum NodeKind {
    NdAdd, // +
    NdSub, // -
    NdMul, // *
    NdDiv, // /
    NdNum, // number
    NdNeg, // unary =
    NdEq,  // ==
    NdNe,  // !=
    NdLt,  // <
    NdLe,  // <=
    NdGt,  // >
    NdGe,  // >=
}

#[derive(Clone)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: i32,
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
        // ==, !=, <=, >= p.len() > 2 がないとindex out of boundsになる
        if p.len() > 2
            && (p[0..2].eq("==") || p[0..2].eq("!=") || p[0..2].eq("<=") || p[0..2].eq(">="))
        {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: p[0..2].to_string(),
                loc: index,
            });
            *p = &p[2..];
            index += 2;
            continue;
        }

        if c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '('
            || c == ')'
            || c == '<'
            || c == '>'
        {
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

fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Node {
    Node {
        kind: kind,
        lhs: Some(Box::new(lhs)),
        rhs: Some(Box::new(rhs)),
        val: 0,
    }
}

fn new_unary(kind: NodeKind, lhs: Node) -> Node {
    Node {
        kind: kind,
        lhs: Some(Box::new(lhs)),
        rhs: None,
        val: 0,
    }
}

fn new_num(val: i32) -> Node {
    Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        val: val,
    }
}

//
fn expr(tokens: &mut Vec<Token>, input: &str) -> Node {
    return equality(tokens, input);
}

fn equality(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut node = relational(tokens, input);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "==" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdEq, node.clone(), relational(tokens, input));
            continue;
        }
        if t.str == "!=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdNe, node.clone(), relational(tokens, input));
            continue;
        }
        break;
    }
    return node;
}

fn relational(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut node = add(tokens, input);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "<" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdLt, node.clone(), add(tokens, input));
            continue;
        }
        if t.str == "<=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdLe, node.clone(), add(tokens, input));
            continue;
        }
        if t.str == ">" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdGt, node.clone(), add(tokens, input));
            continue;
        }
        if t.str == ">=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdGe, node.clone(), add(tokens, input));
            continue;
        }
        break;
    }
    return node;
}

fn add(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut node = mul(tokens, input); // この辺の引数の渡し方は合っているのか？
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "+" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdAdd, node.clone(), mul(tokens, input));
            continue;
        }
        if t.str == "-" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdSub, node.clone(), mul(tokens, input));
            continue;
        }
        break;
    }
    return node;
}

fn mul(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut node = unary(tokens, input); // ここが&ありかなしかで変わる
    while !tokens.is_empty() {
        // これがemptyになるかどうかでやるのはダメかと思ったけど案外悪くないのか？いや、悪いか。1+1とかだと無限ループになりそう。breakを入れるとよさげ
        // breakを入れたら動いたというだけでis_emptyが適切かどうかは要確認
        let t = &tokens[0];
        if t.str == "*" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdMul, node.clone(), unary(tokens, input)); // ()をサポートするようになったら、ここもトークンを一つ渡すのではなくtokensを渡すようになるはず
            continue;
        }
        if t.str == "/" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdDiv, node.clone(), unary(tokens, input));
            continue;
        }
        break;
    }
    return node;
}

fn unary(tokens: &mut Vec<Token>, input: &str) -> Node {
    // これがemptyになるかどうかでやるのはダメかと思ったけど案外悪くないのか？いや、悪いか。1+1とかだと無限ループになりそう。breakを入れるとよさげ
    // breakを入れたら動いたというだけでis_emptyが適切かどうかは要確認
    let t = &tokens[0];
    if t.str == "+" {
        tokens.remove(0);
        return unary(tokens, input); // ()をサポートするようになったら、ここもトークンを一つ渡すのではなくtokensを渡すようになるはず。&tokensはだめなんだ
    }
    if t.str == "-" {
        tokens.remove(0);
        return new_unary(NodeKind::NdNeg, unary(tokens, input));
    }
    return primary(tokens, input);
}

// ちゃんと正しくremoveしてトークンを勧められているのかは、動くからみたいな感じになっていてよくない

fn primary(tokens: &mut Vec<Token>, input: &str) -> Node {
    if tokens[0].str == "(" {
        tokens.remove(0);
        let node = expr(tokens, input);
        skip(tokens, ")", input);
        return node;
    }
    match tokens[0].kind {
        TokenKind::TkNum => {
            let num = new_num(tokens[0].val);
            tokens.remove(0); // ここで消費すべきだった
            return num;
        }
        _ => error_tok(&tokens[0], "expected number", input),
    }
}

fn gen_expr(node: Node) {
    match node.kind {
        NodeKind::NdNum => {
            println!("  mov x0, {}", node.val);
            return;
        }
        NodeKind::NdNeg => {
            gen_expr(*(node.lhs).unwrap()); // lhsはあるはず
            println!("  neg x0, x0");
            return;
        }
        _ => {}
    }

    // node.lhs, node.rhsがあれば、それを再帰的に呼び出す
    if let Some(lhs) = node.lhs {
        gen_expr(*lhs);
        println!("  str x0, [sp, -16]!"); // 16バイトアラインメントしているのか？
    }
    if let Some(rhs) = node.rhs {
        gen_expr(*rhs);
    }
    println!("  ldr x1, [sp], 16");
    // 今、rhsの計算結果がx0, lhsの計算結果がx1に入っている

    match node.kind {
        NodeKind::NdAdd => {
            println!("  add x0, x1, x0");
        }
        NodeKind::NdSub => {
            println!("  sub x0, x1, x0");
        }
        NodeKind::NdMul => {
            println!("  mul x0, x1, x0");
        }
        NodeKind::NdDiv => {
            // x0が0だった場合は未定義になりそう？そのためにはcmpとか？一旦後で
            println!("  cbz x0, error"); // x0が0の場合はエラー処理に飛ぶ
            println!("  sdiv x0, x1, x0");
        }
        NodeKind::NdEq => {
            println!("  cmp x1, x0");
            println!("  cset x0, eq");
        }
        NodeKind::NdNe => {
            println!("  cmp x1, x0");
            println!("  cset x0, ne");
        }
        NodeKind::NdLt => {
            println!("  cmp x1, x0");
            println!("  cset x0, lt");
        }
        NodeKind::NdLe => {
            println!("  cmp x1, x0");
            println!("  cset x0, le");
        }
        NodeKind::NdGt => {
            println!("  cmp x1, x0");
            println!("  cset x0, gt");
        }
        NodeKind::NdGe => {
            println!("  cmp x1, x0");
            println!("  cset x0, ge");
        }
        _ => eprintln!("invalid node kind"),
    }
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
    let node = expr(&mut tokens, input_copy);
    // println!("{:?}", tokens);

    // 最初の文字は数値
    println!(".global _main");
    println!("_main:");
    gen_expr(node);
    println!("  b end");

    // ゼロ徐算の場合のエラー処理
    println!("error:");
    println!("  mov x0, 1");
    println!("  b end"); // これじゃあただ正常に計算結果が1なのか、エラーが1なのかわからないのでは？まあ今は動くのでよしとする

    println!("end:");
    println!("  ret");
}

fn skip(tokens: &mut Vec<Token>, op: &str, input: &str) -> bool {
    if tokens.is_empty() {
        eprintln!("{}", input);
        eprintln!("{}^", " ".repeat(input.len()));
        eprintln!("expected {}", op);
        std::process::exit(1);
    }
    if tokens[0].str != op {
        error_tok(&tokens[0], format!("expected {}", op).as_str(), input); // as_str()は&strに変換するためのもので、to_string()はStringに変換するためのもの
    }
    tokens.remove(0);
    return true;
}

fn parse_number(p: &mut &str) -> String {
    let num: String = p.chars().take_while(|c| c.is_digit(10)).collect();
    *p = &p[num.len()..]; // これは関数の外に出した方が明示的に書きやすいかも？
    return num;
}

// 嘘だけどNodeを返すと書いている
fn error_tok(t: &Token, msg: &str, input: &str) -> Node {
    eprintln!("{}", input);
    eprintln!("{}^", " ".repeat(t.loc));
    eprintln!("{}", msg);
    std::process::exit(1);
}
