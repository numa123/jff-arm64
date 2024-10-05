use std::mem::swap;

use crate::tokenize::{consume, error_tok, skip};
use crate::types::{
    add_type, is_integer, is_pointer, is_typename, new_array, new_char, new_int, new_ptr_to, Node,
    NodeKind, Token, TokenKind, Type, Var,
};

pub static mut GLOBALS: Vec<Var> = Vec::new();

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

fn new_var_node(var: Var) -> Node {
    let mut node = new_node(NodeKind::NdVar);
    node.var = Some(Box::new(var));
    return node;
}

fn new_block(block_body: Vec<Node>) -> Node {
    let mut node = new_node(NodeKind::NdBlock);
    node.block_body = block_body;
    return node;
}

fn new_var(v: &mut Vec<Var>, name: &str, ty: Type, is_arg_def: bool) -> Var {
    let var = Var {
        name: name.to_string(),
        offset: v.len(),
        def_arg: is_arg_def,
        ty: ty,
        is_func: false,
        stmts: Vec::new(),
        variables: Vec::new(),
        args: Vec::new(),
        gval: None,
        str: None,
    };
    v.push(var.clone());
    return var;
}

// declaration_specifier = "int" | "char"
fn declaration_specifier(tokens: &mut Vec<Token>, input: &str) -> Type {
    if tokens[0].str == "char" {
        tokens.remove(0);
        return new_char();
    }
    skip(tokens, "int", input);
    return new_int();
}

fn type_chain(tokens: &mut Vec<Token>, input: &str, ty: Type) -> Type {
    let mut ty = ty;
    while consume(tokens, "*") {
        ty = new_ptr_to(ty);
    }
    if tokens[0].kind != TokenKind::TkIdent {
        error_tok(&tokens[0], "expected identifier", input);
    }
    return ty;
}

fn declaration(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let base_ty = declaration_specifier(tokens, input);
    let mut body: Vec<Node> = Vec::new();

    while tokens[0].str != ";" {
        let mut ty = type_chain(tokens, input, base_ty.clone());
        if !tokens[0].kind.eq(&TokenKind::TkIdent) {
            error_tok(&tokens[0], "expected identifier", input);
        }
        let name = tokens[0].str.clone();
        tokens.remove(0);
        // array
        if tokens[0].str == "[" {
            tokens.remove(0);
            let num = tokens[0].val;
            tokens.remove(0);
            skip(tokens, "]", input);
            ty = new_array(ty, num as usize);
        }
        let var = create_var(v, name.as_str(), ty, false);

        if tokens[0].str == "=" {
            tokens.remove(0);
            let node = new_binary(
                NodeKind::NdAssign,
                new_var_node(var.clone()),
                expr(tokens, input, v),
            );
            body.push(new_unary(NodeKind::NdExprStmt, node));
        }

        if tokens[0].str == "," {
            tokens.remove(0);
        }
    }

    let mut node = new_node(NodeKind::NdBlock);
    node.block_body = body;
    skip(tokens, ";", input);
    return node;
}

//
// function
//
fn func_param(tokens: &mut Vec<Token>, input: &str, func: &mut Var) {
    let base_ty = declaration_specifier(tokens, input);
    let ty = type_chain(tokens, input, base_ty);
    // int add(int 1, int 2){}のような関数定義をエラーに
    if !tokens[0].kind.eq(&TokenKind::TkIdent) {
        error_tok(&tokens[0], "expected identifier", input);
    }
    let var = new_var(&mut func.variables, tokens[0].str.as_str(), ty, true);
    tokens.remove(0);
    func.variables.push(var.clone());
    func.args.push(new_var_node(var));

    while tokens[0].str == "," {
        tokens.remove(0);
        let base_ty = declaration_specifier(tokens, input);
        let ty = type_chain(tokens, input, base_ty);
        if !tokens[0].kind.eq(&TokenKind::TkIdent) {
            error_tok(&tokens[0], "expected identifier", input);
        }
        let var = new_var(&mut func.variables, tokens[0].str.as_str(), ty, true);
        tokens.remove(0);
        func.variables.push(var.clone());
        func.args.push(new_var_node(var));
    }
}

// ゴミコード
fn function_or_variable_declaration(tokens: &mut Vec<Token>, input: &str) -> Var {
    let base_ty = declaration_specifier(tokens, input); // int
    let ty = type_chain(tokens, input, base_ty);

    let mut var = Var {
        name: tokens[0].str.clone(),
        ty: ty,
        offset: unsafe { GLOBALS.len() },
        def_arg: false,
        is_func: false,
        stmts: Vec::new(),
        variables: Vec::new(), // あとで使うけど、今は一旦int main()だけ書けるようにするか
        args: Vec::new(),
        gval: None,
        str: None,
    };
    tokens.remove(0);

    if tokens[0].str == "(" {
        var.is_func = true;
        skip(tokens, "(", input);

        if tokens[0].str == ")" {
            skip(tokens, ")", input);
            if var.variables.iter().filter(|v| v.def_arg == true).count() > 8 {
                error_tok(&tokens[0], "too many arguments", input);
            }
            skip(tokens, "{", input); // compound-stmtのEBNF忘れてた
            let block = compound_stmt(tokens, input, &mut var.variables);
            var.stmts = block.block_body;
            unsafe {
                GLOBALS.push(var.clone());
            }
            return var;
        }

        func_param(tokens, input, &mut var);
        skip(tokens, ")", input);
        // func.variablesの中でdef_arg: trueのものの数が8個を超えたらエラーを出す
        if var.variables.iter().filter(|v| v.def_arg == true).count() > 8 {
            error_tok(&tokens[0], "too many arguments", input);
        }
        skip(tokens, "{", input); // compound-stmtのEBNF忘れてた
        let block = compound_stmt(tokens, input, &mut var.variables);
        var.stmts = block.block_body;
        unsafe {
            GLOBALS.push(var.clone());
        }

        return var;
    }

    // varって、valとかないのか
    let node = new_node(NodeKind::NdBlock);
    if tokens[0].str == "=" {
        tokens.remove(0);
        var.gval = Some(tokens[0].val);
        tokens.remove(0);
        unsafe {
            GLOBALS.push(var.clone());
        }
        var.stmts = node.block_body;
        skip(tokens, ";", input);
        return var;
    }

    var.gval = Some(0);
    unsafe {
        GLOBALS.push(var.clone());
    }
    skip(tokens, ";", input);
    return var;
}

//
// node
//

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

        node.init = if is_typename(tokens[0].clone()) {
            Some(Box::new(declaration(tokens, input, v)))
        } else {
            Some(Box::new(expr_stmt(tokens, input, v)))
        };
        //  Some(Box::new(expr_stmt(tokens, input, v)));
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
        let mut node = if is_typename(tokens[0].clone()) {
            declaration(tokens, input, v)
        } else {
            stmt(tokens, input, v)
        };
        add_type(&mut node);
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
    let node = and(tokens, input, v);
    if tokens[0].str == "=" {
        tokens.remove(0);
        return new_binary(NodeKind::NdAssign, node, assign(tokens, input, v));
    }
    return node;
}

fn and(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = equality(tokens, input, v);
    while !tokens.is_empty() {
        if tokens[0].str == "&&" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdAnd, node, equality(tokens, input, v));
            continue;
        }
        break;
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
    // normalize num + ptr to ptr + num
    if is_integer(lhs_ty) && is_pointer(rhs_ty) {
        swap(lhs, rhs);
    }
    // pointer + num
    let r = new_binary(
        NodeKind::NdMul,
        rhs.clone(),
        new_num(lhs.clone().ty.unwrap().ptr_to.unwrap().size as i32), // またcloneしている
    ); // chibiccではrhsを変更している
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
        n.ty = Some(new_int()); // ???
        return new_binary(
            NodeKind::NdDiv,
            n,
            new_num(lhs.clone().ty.unwrap().ptr_to.unwrap().size as i32),
        );
    }
    error_tok(&tokens[0], "invalid operands", input)
}

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
        if t.str == "%" {
            tokens.remove(0);
            node = new_binary(NodeKind::NdMod, node.clone(), unary(tokens, input, v));
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
    return postfix(tokens, input, v);
}

fn postfix(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    let mut node = primary(tokens, input, v);
    if tokens[0].str == "[" {
        tokens.remove(0);
        let mut idx = expr(tokens, input, v);
        skip(tokens, "]", input);
        node = new_add(tokens, input, &mut node, &mut idx);
        node = new_unary(NodeKind::NdDeref, node);
    }
    return node;
}

fn primary(tokens: &mut Vec<Token>, input: &str, v: &mut Vec<Var>) -> Node {
    if tokens[0].str == "(" {
        tokens.remove(0);
        let node = expr(tokens, input, v);
        skip(tokens, ")", input);
        return node;
    }
    if tokens[0].str == "sizeof" {
        tokens.remove(0);
        let mut node = unary(tokens, input, v);
        add_type(&mut node);
        return new_num(node.ty.unwrap().size as i32);
    }
    match tokens[0].kind {
        TokenKind::TkNum => {
            let num = new_num(tokens[0].val);
            tokens.remove(0); // ここでremoveしないでバグったことがあった
            return num;
        }
        // tokens[0] は、str = "hoge"
        TokenKind::TkStr => {
            let var = Var {
                name: format!("lC{}", unsafe { GLOBALS.len() }),
                ty: new_array(new_char(), tokens[0].str.len()),
                offset: 0, // ここはどうでもいい
                def_arg: false,
                is_func: false,
                stmts: Vec::new(),
                variables: Vec::new(),
                args: Vec::new(),
                gval: None,
                str: Some(tokens[0].str.clone()),
            };
            unsafe {
                GLOBALS.push(var.clone());
            }
            tokens.remove(0);
            return new_var_node(var);
        }
        TokenKind::TkIdent => {
            // function call
            if tokens.len() >= 2 && tokens[1].str == "(" {
                let mut node = new_node(NodeKind::NdFuncCall);
                node.func_name = tokens[0].str.clone();
                tokens.remove(0);

                let mut args: Vec<Node> = Vec::new(); // 引数のノードを格納する
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
            // ここでidentが見つからないはずはない。declarationで追加しているから。という想定
            let var = if let Some(v) = find_var(v, tokens[0].str.as_str()) {
                v
            } else if let Some(v) = find_var(&unsafe { GLOBALS.clone() }, tokens[0].str.as_str()) {
                // clone祭りすぎ
                v
            } else {
                error_tok(&tokens[0], "undefined variable", input);
                panic!();
            };
            let ident = new_var_node(var);
            tokens.remove(0);
            return ident;
        }
        _ => error_tok(&tokens[0], "expected number", input),
    }
}

// 変数がすでに存在する場合、その変数を返す
fn find_var(var: &Vec<Var>, name: &str) -> Option<Var> {
    return var.iter().find(|v| v.name == name).cloned();
}

// no create function
fn create_var(v: &mut Vec<Var>, name: &str, ty: Type, is_arg_def: bool) -> Var {
    let nv = Var {
        name: name.to_string(),
        ty: ty,
        offset: v.len(),
        def_arg: is_arg_def,
        is_func: false,
        stmts: Vec::new(),
        variables: Vec::new(),
        args: Vec::new(),
        gval: None,
        str: None,
    };
    v.push(nv.clone());
    return nv;
}

pub fn parse(tokens: &mut Vec<Token>, input: &str) -> Vec<Var> {
    while !tokens.is_empty() {
        function_or_variable_declaration(tokens, input);
    }
    return unsafe { GLOBALS.clone() }; // 絶対無駄
}
