use crate::tokenize::{error_tok, skip};
use crate::types::{Node, NodeKind, Token, TokenKind, Var};

//
// Node
//
pub static mut VARIABLES: Vec<Var> = Vec::new();

pub static mut HASFUNCCALL: bool = false;

fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Node {
    let mut node = new_node(kind);
    node.lhs = Some(Box::new(lhs));
    node.rhs = Some(Box::new(rhs));
    return node;
}

fn new_unary(kind: NodeKind, lhs: Node) -> Node {
    let mut node = new_node(kind);
    node.lhs = Some(Box::new(lhs));
    return node;
}

fn new_num(val: i32) -> Node {
    let mut node = new_node(NodeKind::NdNum);
    node.val = val;
    return node;
}

fn new_var(var: Var) -> Node {
    let mut node = new_node(NodeKind::NdVar);
    node.var = Some(Box::new(var));
    return node;
}

fn new_block(block_body: Vec<Node>) -> Node {
    let mut node = new_node(NodeKind::NdBlock);
    node.block_body = block_body;
    return node;
}

fn new_node(kind: NodeKind) -> Node {
    Node {
        kind: kind,
        lhs: None,
        rhs: None,
        val: 0,
        var: None,
        block_body: Vec::new(),
        cond: None,
        then: None,
        els: None,
        init: None,
        inc: None,
        funcname: "".to_string(), // String::new()でもいいのでは。内部的には同じか？
    }
}

//
fn stmt(tokens: &mut Vec<Token>, input: &str) -> Node {
    if tokens[0].str == "return" {
        tokens.remove(0);
        // println!("returnが呼ばれた時: {:?}", tokens);
        let node = new_unary(NodeKind::NdReturn, expr(tokens, input));
        // println!("returnが呼ばれたあと: {:?}", tokens);
        skip(tokens, ";", input);
        return node;
    }
    if tokens[0].str == "{" {
        tokens.remove(0);
        let node = compound_stmt(tokens, input);
        return node;
    }
    if tokens[0].str == "if" {
        let mut node = new_node(NodeKind::NdIf);
        tokens.remove(0);
        skip(tokens, "(", input);
        let cond = expr(tokens, input);
        skip(tokens, ")", input);
        let then = stmt(tokens, input);
        node.cond = Some(Box::new(cond));
        node.then = Some(Box::new(then));
        if tokens.len() != 0 && tokens[0].str == "else" {
            // elseがない場合、index out of boundsになる
            tokens.remove(0);
            let els = stmt(tokens, input);
            node.els = Some(Box::new(els));
        }
        return node;
    }
    if tokens[0].str == "for" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        let init = expr_stmt(tokens, input);
        node.init = Some(Box::new(init));
        if tokens[0].str != ";" {
            let cond = expr(tokens, input);
            node.cond = Some(Box::new(cond));
        }
        skip(tokens, ";", input);
        if tokens[0].str != ")" {
            let inc = expr(tokens, input);
            node.inc = Some(Box::new(inc));
        }
        skip(tokens, ")", input);
        let then = stmt(tokens, input);
        node.then = Some(Box::new(then));
        return node;
    }
    if tokens[0].str == "while" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        let cond = expr(tokens, input);
        node.cond = Some(Box::new(cond));
        skip(tokens, ")", input);
        let then = stmt(tokens, input);
        node.then = Some(Box::new(then));
        return node;
    }
    return expr_stmt(tokens, input);
}

fn compound_stmt(tokens: &mut Vec<Token>, input: &str) -> Node {
    let mut block_body = Vec::new();
    while !(tokens[0].str == "}") {
        let node = stmt(tokens, input);
        block_body.push(node);
        continue;
    }
    let node = new_block(block_body);
    skip(tokens, "}", input);
    return node;
}

fn expr_stmt(tokens: &mut Vec<Token>, input: &str) -> Node {
    if tokens.len() != 0 && tokens[0].str == ";" {
        let node = new_block(Vec::new());
        skip(tokens, ";", input);
        return node;
    }
    let node = new_unary(NodeKind::NdExprStmt, expr(tokens, input));
    skip(tokens, ";", input);
    return node;
}

fn expr(tokens: &mut Vec<Token>, input: &str) -> Node {
    return assign(tokens, input);
}

fn assign(tokens: &mut Vec<Token>, input: &str) -> Node {
    let node = equality(tokens, input);
    if tokens[0].str == "=" {
        tokens.remove(0);
        return new_binary(NodeKind::NdAssign, node, assign(tokens, input));
    }
    return node;
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
        TokenKind::TkIdent => {
            // funccall
            if tokens.len() >= 2 && tokens[1].str == "(" {
                let mut node = new_node(NodeKind::NdFuncCall);
                node.funcname = tokens[0].str.clone();
                tokens.remove(0); // 次は"("の予定
                skip(tokens, "(", input);
                skip(tokens, ")", input);
                unsafe {
                    HASFUNCCALL = true;
                };
                return node;
            }

            let var: Var;
            unsafe {
                var = if let Some(v) = VARIABLES.iter().find(|v| v.name == tokens[0].str) {
                    v.clone()
                } else {
                    let nv = Var {
                        name: tokens[0].str.clone(),
                        offset: VARIABLES.len(), // ここで一位に決める
                    };
                    VARIABLES.push(nv.clone());
                    nv
                };
            }
            let ident = new_var(var);
            tokens.remove(0);
            return ident;
        }
        _ => error_tok(&tokens[0], "expected number", input),
    }
}

pub fn parse(tokens: &mut Vec<Token>, input: &str) -> Vec<Node> {
    let mut stmts = Vec::new();
    while !tokens.is_empty() {
        stmts.push(stmt(tokens, input));
    }
    return stmts;
}
