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

#[derive(Clone, PartialEq)]
enum NodeKind {
    NdAdd,
    NdSub,
    NdMul,
    NdDiv,
    NdNum,
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
        if c == '+' || c == '-' || c == '*' || c == '/' {
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

// fn new_node(kind: NodeKind) -> Node {
//     Node {
//         kind: kind,
//         lhs: None,
//         rhs: None,
//         val: 0,
//     }
// }

fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Node {
    Node {
        kind: kind,
        lhs: Some(Box::new(lhs)),
        rhs: Some(Box::new(rhs)),
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

// 嘘だけどNodeを返すと書いている
fn error_tok(t: &Token, input: &str) -> Node {
    eprintln!("{}", input);
    eprintln!("{}^", " ".repeat(t.loc));
    eprintln!("expected a number");
    std::process::exit(1);
}

//
fn expr(tokens: &mut Vec<Token>, input: &str) -> Node {
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
    }
    return node;
}

fn mul(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut node = primary(&tokens[0], input);
    tokens.remove(0);
    while !tokens.is_empty() {
        // これがemptyになるかどうかでやるのはダメかと思ったけど案外悪くないのか？いや、悪いか。1+1とかだと無限ループになりそう。breakを入れるとよさげ
        let t = &tokens[0];
        if t.str == "*" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdMul, node.clone(), primary(&tokens[0], input)); // ()をサポートするようになったら、ここもトークンを一つ渡すのではなくtokensを渡すようになるはず
            tokens.remove(0);
            continue;
        }
        if t.str == "/" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdDiv, node.clone(), primary(&tokens[0], input));
            tokens.remove(0);
            continue;
        }
        break;
    }
    return node;
}

fn primary(t: &Token, input: &str) -> Node {
    match t.kind {
        TokenKind::TkNum => new_num(t.val),
        _ => error_tok(t, input),
    }
}

fn gen_expr(node: Node) {
    if node.kind == NodeKind::NdNum {
        println!("  mov x0, {}", node.val);
        return;
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
        _ => eprintln!("invalid node kind"), // 今はprimaryだけ
    }
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
    let node = expr(&mut tokens, input_copy);
    // println!("{:?}", tokens);

    // 最初の文字は数値
    println!(".global _main");
    println!("_main:");
    gen_expr(node);
    println!("  b end"); // これじゃあただ正常に計算結果が1なのか、エラーが1なのかわからない

    // ゼロ徐算の場合のエラー処理
    println!("error:");
    println!("  mov x0, 1");
    println!("  b end"); // これじゃあただ正常に計算結果が1なのか、エラーが1なのかわからない

    println!("end:");
    println!("  ret");
}

// fn get_number(t: &Token, input: &str) -> i32 {
//     match t.kind {
//         TokenKind::TkNum => t.val,
//         _ => {
//             eprintln!("{}", input);
//             eprintln!("{}^", " ".repeat(t.loc));
//             eprintln!("expected a number");
//             std::process::exit(1);
//         }
//     }
// }
