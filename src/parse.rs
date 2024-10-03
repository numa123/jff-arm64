use std::mem::swap;

use crate::tokenize::{error_tok, skip};
use crate::types::{
    add_type, is_integer, is_pointer, Function, Node, NodeKind, Token, TokenKind, Type, TypeKind,
    Var,
};

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
        ty: None,
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
fn function(tokens: &mut Vec<Token>, input: &str) -> Function {
    skip(tokens, "int", input);
    let mut func = Function {
        name: tokens[0].str.clone(),
        stmts: Vec::new(),
        variables: Vec::new(), // あとで使うけど、今は一旦int main()だけ書けるようにするか
        args: Vec::new(),
    };
    tokens.remove(0);
    skip(tokens, "(", input);

    if tokens[0].str == ")" {
        skip(tokens, ")", input);
        return func;
    }

    skip(tokens, "int", input);
    // int add(int 1, int 2){}のような関数定義をエラーに
    if !tokens[0].kind.eq(&TokenKind::TkIdent) {
        error_tok(&tokens[0], "expected identifier", input);
    }
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
        if !tokens[0].kind.eq(&TokenKind::TkIdent) {
            error_tok(&tokens[0], "expected identifier", input);
        }
        let var = Var {
            name: tokens[0].str.clone(),
            offset: func.variables.len(),
            def_arg: true,
        };
        tokens.remove(0);
        func.variables.push(var.clone());
        func.args.push(new_var(var));
    }

    // func.variablesの中でdef_arg: trueのものの数が8個を超えたらエラーを出す
    if func.variables.iter().filter(|v| v.def_arg == true).count() > 8 {
        error_tok(&tokens[0], "too many arguments", input);
    }

    skip(tokens, ")", input);
    return func;
}

//
// node
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

    // if:  "if" "(" expr ")" stmt ("else" stmt)?
    if tokens[0].str == "if" {
        let mut node = new_node(NodeKind::NdIf);
        tokens.remove(0);
        skip(tokens, "(", input);
        node.cond = Some(Box::new(expr(tokens, input, v)));
        skip(tokens, ")", input);
        node.then = Some(Box::new(stmt(tokens, input, v)));
        // elseがない場合、index out of boundsにならないように、tokens.len() != 0を入れている
        if tokens.len() != 0 && tokens[0].str == "else" {
            tokens.remove(0);
            node.els = Some(Box::new(stmt(tokens, input, v)));
        }
        return node;
    }

    // for: "for" "(" expr_stmt expr? ";" expr? ")" stmt
    if tokens[0].str == "for" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        node.init = Some(Box::new(expr_stmt(tokens, input, v)));
        if tokens[0].str != ";" {
            node.cond = Some(Box::new(expr(tokens, input, v)));
        }
        skip(tokens, ";", input);
        if tokens[0].str != ")" {
            node.inc = Some(Box::new(expr(tokens, input, v)));
        }
        skip(tokens, ")", input);
        node.then = Some(Box::new(stmt(tokens, input, v)));
        return node;
    }

    // while: "while" "(" expr ")" stmt ==> for (; expr; ) stmt
    if tokens[0].str == "while" {
        let mut node = new_node(NodeKind::NdFor);
        tokens.remove(0);
        skip(tokens, "(", input);
        node.cond = Some(Box::new(expr(tokens, input, v)));
        skip(tokens, ")", input);
        node.then = Some(Box::new(stmt(tokens, input, v)));
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
        // add_type(&mut node); // chibiccには書いてあるけど、なくても動くので一旦コメントアウト
        block_body.push(node);
        continue;
    }
    let node = new_block(block_body);
    skip(tokens, "}", input);
    return node;
}

fn expr_stmt(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    // null statement
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

//
// pointer arithmetic
//
// 副作用すごそうだけど、動いてる。
fn new_add(tokens: &mut Vec<Token>, input: &str, lhs: &mut Node, rhs: &mut Node) -> Node {
    add_type(lhs);
    add_type(rhs);
    let lhs_ty = lhs.ty.as_ref().unwrap();
    let rhs_ty = rhs.ty.as_ref().unwrap();
    // num + num
    if is_integer(lhs_ty) && is_integer(rhs_ty) {
        return new_binary(NodeKind::NdAdd, lhs.clone(), rhs.clone());
    }
    // cannot add pointer and pointer
    if is_pointer(lhs_ty) && is_pointer(rhs_ty) {
        error_tok(
            &tokens[0],
            "invalid operands: both of operands are pointer", // これでよいのか？
            input,
        );
    }
    // normalize to ptr + num
    if is_integer(lhs_ty) && is_pointer(rhs_ty) {
        swap(lhs, rhs);
    }
    // pointer + num
    let r = new_binary(NodeKind::NdMul, rhs.clone(), new_num(8)); // chibiccではrhsを変更している
    return new_binary(NodeKind::NdAdd, lhs.clone(), r);
}

fn new_sub(tokens: &mut Vec<Token>, input: &str, lhs: &mut Node, rhs: &mut Node) -> Node {
    add_type(lhs);
    add_type(rhs);
    let lhs_ty = lhs.ty.as_ref().unwrap();
    let rhs_ty = rhs.ty.as_ref().unwrap();
    // num - num
    if is_integer(lhs_ty) && is_integer(rhs_ty) {
        return new_binary(NodeKind::NdSub, lhs.clone(), rhs.clone());
    }
    // pointer - num
    if is_pointer(lhs_ty) && is_integer(rhs_ty) {
        let r = new_binary(NodeKind::NdMul, rhs.clone(), new_num(8));
        let mut n = new_binary(NodeKind::NdSub, lhs.clone(), r);
        n.ty = Some(lhs.ty.as_ref().unwrap().clone());
        return n;
    }
    // pointer - pointer // もうわからないから写経しかすることない。泣きたい
    if is_pointer(lhs_ty) && is_pointer(rhs_ty) {
        let mut n = new_binary(NodeKind::NdSub, lhs.clone(), rhs.clone());
        n.ty = Some(Type {
            kind: TypeKind::TyInt,
            ptr_to: None,
        });
        return new_binary(NodeKind::NdDiv, n, new_num(8));
    }
    error_tok(&tokens[0], "invalid operands", input)
}
//

//
fn add(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = mul(tokens, input, v);
    while !tokens.is_empty() {
        let t = &tokens[0];
        if t.str == "+" {
            tokens.remove(0);
            let mut m = mul(tokens, input, v);
            node = new_add(tokens, input, &mut node, &mut m);
            continue;
        }
        if t.str == "-" {
            tokens.remove(0);
            let mut m = mul(tokens, input, v);
            node = new_sub(tokens, input, &mut node, &mut m);
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
    if t.str == "&" {
        tokens.remove(0);
        return new_unary(NodeKind::NdAddr, unary(tokens, input, v));
    }
    if t.str == "*" {
        tokens.remove(0);
        return new_unary(NodeKind::NdDeref, unary(tokens, input, v));
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

                // 引数なしのパターン
                if tokens[0].str == ")" {
                    skip(tokens, ")", input);
                    return node;
                }
                // 引数があるパターン
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
