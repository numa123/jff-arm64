use crate::tokenize::{error_tok, skip};
use crate::types::{Function, Node, NodeKind, Token, TokenKind, Var};

// pub static mut VARIABLES: Vec<Var> = Vec::new();

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
        func_name: String::new(),
        args: Vec::new(),
    }
}

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

//
// function
//
// 今は、int main(1, 2) {} みたいな宣言も受け付けれてしまう。変数名のみにしたいところではある
fn function(tokens: &mut Vec<Token>, input: &str) -> Function {
    skip(tokens, "int", input);
    let mut func = Function {
        name: tokens[0].str.clone(),
        stmts: Vec::new(),
        variables: Vec::new(), // あとで使うけど、今は一旦int main()だけ書けるようにするか
        args: Vec::new(),
    };
    // let mut variables = Vec::new();
    tokens.remove(0); // 名前を消費
    skip(tokens, "(", input);

    if tokens[0].str == ")" {
        skip(tokens, ")", input);
        return func;
    }

    skip(tokens, "int", input);
    let var = Var {
        name: tokens[0].str.clone(),
        offset: func.variables.len(),
        def_arg: true,
    };
    tokens.remove(0);
    func.variables.push(var.clone());
    func.args.push(new_var(var));

    while tokens[0].str == "," {
        tokens.remove(0);
        skip(tokens, "int", input);
        let var = Var {
            name: tokens[0].str.clone(),
            offset: func.variables.len(),
            def_arg: true,
        };
        tokens.remove(0);
        func.variables.push(var.clone());

        let arg = new_var(var);
        func.args.push(arg);
    }

    // func.variablesの中でdef_arg: trueのものの数が8個を超えたらエラーを出す
    if func.variables.iter().filter(|v| v.def_arg == true).count() > 8 {
        error_tok(&tokens[0], "too many arguments", input);
    }

    skip(tokens, ")", input);
    return func;
}

//
// nodenize functions
//

fn declaration(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    skip(tokens, "int", input);
    return expr_stmt(tokens, input, v);
}

fn stmt(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    // return
    if tokens[0].str == "return" {
        tokens.remove(0);
        let node = new_unary(NodeKind::NdReturn, expr(tokens, input, v));
        skip(tokens, ";", input);
        return node;
    }

    // compound statement
    if tokens[0].str == "{" {
        tokens.remove(0);
        let node = compound_stmt(tokens, input, v);
        return node;
    }

    // if
    if tokens[0].str == "if" {
        let mut node = new_node(NodeKind::NdIf);
        tokens.remove(0);
        skip(tokens, "(", input);
        let cond = expr(tokens, input, v);
        skip(tokens, ")", input);
        let then = stmt(tokens, input, v);
        node.cond = Some(Box::new(cond));
        node.then = Some(Box::new(then));
        // elseがない場合、index out of boundsにならないように、tokens.len() != 0を入れている
        if tokens.len() != 0 && tokens[0].str == "else" {
            tokens.remove(0);
            let els = stmt(tokens, input, v);
            node.els = Some(Box::new(els));
        }
        return node;
    }

    // for
    if tokens[0].str == "for" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        let init = expr_stmt(tokens, input, v);
        node.init = Some(Box::new(init));
        if tokens[0].str != ";" {
            let cond = expr(tokens, input, v);
            node.cond = Some(Box::new(cond));
        }
        skip(tokens, ";", input);
        if tokens[0].str != ")" {
            let inc = expr(tokens, input, v);
            node.inc = Some(Box::new(inc));
        }
        skip(tokens, ")", input);
        let then = stmt(tokens, input, v);
        node.then = Some(Box::new(then));
        return node;
    }

    // while
    // forを再利用する
    if tokens[0].str == "while" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        let cond = expr(tokens, input, v);
        node.cond = Some(Box::new(cond));
        skip(tokens, ")", input);
        let then = stmt(tokens, input, v);
        node.then = Some(Box::new(then));
        return node;
    }
    return expr_stmt(tokens, input, v);
}

fn compound_stmt(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut block_body = Vec::new();
    while !(tokens[0].str == "}") {
        // int declaration
        let node = if tokens[0].str == "int" {
            declaration(tokens, input, v)
        } else {
            stmt(tokens, input, v)
        };
        block_body.push(node);
        continue;
    }
    let node = new_block(block_body);
    skip(tokens, "}", input);
    return node;
}

fn expr_stmt(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    if tokens.len() != 0 && tokens[0].str == ";" {
        let node = new_block(Vec::new());
        skip(tokens, ";", input);
        return node;
    }
    let node = new_unary(NodeKind::NdExprStmt, expr(tokens, input, v));
    skip(tokens, ";", input);
    return node;
}

fn expr(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    return assign(tokens, input, v);
}

fn assign(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let node = equality(tokens, input, v);
    if tokens[0].str == "=" {
        tokens.remove(0);
        return new_binary(NodeKind::NdAssign, node, assign(tokens, input, v));
    }
    return node;
}

fn equality(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = relational(tokens, input, v);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "==" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdEq, node.clone(), relational(tokens, input, v));
            continue;
        }
        if t.str == "!=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdNe, node.clone(), relational(tokens, input, v));
            continue;
        }
        break;
    }
    return node;
}

fn relational(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = add(tokens, input, v);
    // !tokens.is_empty()が適切かは不明
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "<" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdLt, node.clone(), add(tokens, input, v));
            continue;
        }
        if t.str == "<=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdLe, node.clone(), add(tokens, input, v));
            continue;
        }
        if t.str == ">" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdGt, node.clone(), add(tokens, input, v));
            continue;
        }
        if t.str == ">=" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdGe, node.clone(), add(tokens, input, v));
            continue;
        }
        break;
    }
    return node;
}

fn add(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = mul(tokens, input, v);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "+" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdAdd, node.clone(), mul(tokens, input, v));
            continue;
        }
        if t.str == "-" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdSub, node.clone(), mul(tokens, input, v));
            continue;
        }
        break;
    }
    return node;
}

fn mul(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = unary(tokens, input, v);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "*" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdMul, node.clone(), unary(tokens, input, v));
            continue;
        }
        if t.str == "/" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdDiv, node.clone(), unary(tokens, input, v));
            continue;
        }
        break;
    }
    return node;
}

fn unary(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let t = &tokens[0];
    if t.str == "+" {
        tokens.remove(0);
        return unary(tokens, input, v);
    }
    if t.str == "-" {
        tokens.remove(0);
        return new_unary(NodeKind::NdNeg, unary(tokens, input, v));
    }
    return primary(tokens, input, v);
}

fn primary(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    if tokens[0].str == "(" {
        tokens.remove(0);
        let node = expr(tokens, input, v);
        skip(tokens, ")", input);
        return node;
    }
    match tokens[0].kind {
        TokenKind::TkNum => {
            let num = new_num(tokens[0].val);
            tokens.remove(0); // ここでremoveしないでバグったことがあった
            return num;
        }
        TokenKind::TkIdent => {
            // function call
            if tokens.len() >= 2 && tokens[1].str == "(" {
                let mut node = new_node(NodeKind::NdFuncCall);
                node.func_name = tokens[0].str.clone();
                tokens.remove(0);
                let mut args: Vec<Node> = Vec::new();
                skip(tokens, "(", input);

                // // exprなければnodeを返して終わり
                // // このif文やばい
                if tokens[0].str == ")" {
                    skip(tokens, ")", input);
                    return node;
                }

                // 引数があるパターン
                // exprの次に","がある場合
                //
                // そうだ、トークンじゃなくて、add()とかもあるから、評価値をベースで次にしないといけない
                // tokens[1].strで判定しているのがだめ
                //
                args.push(expr(tokens, input, v)); // 最後の引数

                while tokens.len() >= 2 && tokens[0].str == "," {
                    skip(tokens, ",", input);
                    let arg = expr(tokens, input, v);
                    args.push(arg);
                }

                // 今は引数の個数の上限が8個(x0 ~ x7)なので、それを超えたらエラーを出す
                if args.len() > 8 {
                    error_tok(&tokens[0], "too many arguments", input);
                }

                node.args = args;
                // eprintln!("[skip )]が呼ばれる前 : {:#?}", tokens); // なぜかこの時点で")"が消費されている
                skip(tokens, ")", input);
                return node;
            };

            // variable
            let var: Var;
            var = if let Some(v) = v.iter().find(|v| v.name == tokens[0].str) {
                v.clone()
            } else {
                let nv = Var {
                    name: tokens[0].str.clone(),
                    offset: v.len(), // ここで変数のアドレスを一意に割り当てる
                    def_arg: false,
                };
                v.push(nv.clone());
                nv
            };
            let ident = new_var(var);
            tokens.remove(0);
            return ident;
        }
        _ => error_tok(&tokens[0], "expected number", input),
    }
}

pub fn parse(tokens: &mut Vec<Token>, input: &str) -> Vec<Function> {
    // 今はとにかく1つの関数のみのサポートに取り組む
    // 2つ以上の関数のサポートのためにはVARIABLESを関数ごとに持つ必要がある
    // if tokens[0].str == "int" {
    let mut funcs = Vec::new();
    while !tokens.is_empty() {
        let mut func = function(tokens, input);
        skip(tokens, "{", input); // compound-stmtのEBNF忘れてた
        let block = compound_stmt(tokens, input, &mut func.variables);

        func.stmts = block.block_body;

        funcs.push(func);
    }
    return funcs;
}
