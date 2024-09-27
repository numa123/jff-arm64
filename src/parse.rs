use crate::types::{Node, NodeKind, Token, TokenKind};

pub fn tokenize(p: &mut &str) -> Vec<Token> {
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
pub fn expr(tokens: &mut Vec<Token>, input: &str) -> Node {
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
