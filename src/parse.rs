use std::{cell::RefCell, mem::swap, rc::Rc};

use crate::types::*;

fn new_assign(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdAssign {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_block(body: Vec<Node>) -> Node {
    let mut node = Node {
        kind: NodeKind::NdBlock { body },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn null_stmt() -> Node {
    Node {
        kind: NodeKind::NdBlock { body: Vec::new() },
        ty: None,
    }
}

fn new_expr_stmt(lhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdExprStmt { lhs: Box::new(lhs) },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_return(lhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdReturn { lhs: Box::new(lhs) },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_if(cond: Node, then: Node, els: Option<Node>) -> Node {
    let mut node = Node {
        kind: NodeKind::NdIf {
            cond: Box::new(cond),
            then: Box::new(then),
            els: els.map(Box::new),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_for(init: Node, cond: Option<Node>, inc: Option<Node>, body: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdFor {
            init: Box::new(init),
            cond: cond.map(Box::new),
            inc: inc.map(Box::new),
            body: Box::new(body),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_while(cond: Node, body: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdWhile {
            cond: Box::new(cond),
            body: Box::new(body),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_eq(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdEq {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_ne(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdNe {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_lt(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdLt {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_le(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdLe {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_gt(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdGt {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_ge(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdGe {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_sub(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdSub {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_mul(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdMul {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_div(lhs: Node, rhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdDiv {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_neg(lhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdNeg { lhs: Box::new(lhs) },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_addr(lhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdAddr { lhs: Box::new(lhs) },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_deref(lhs: Node) -> Node {
    let mut node = Node {
        kind: NodeKind::NdDeref { lhs: Box::new(lhs) },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_num(val: isize) -> Node {
    let mut node = Node {
        kind: NodeKind::NdNum { val },
        ty: None,
    };
    add_type(&mut node);
    node
}

fn new_var(var: Rc<RefCell<Var>>) -> Node {
    let mut node = Node {
        kind: NodeKind::NdVar { var },
        ty: None,
    };
    add_type(&mut node);
    node
}

impl Ctx<'_> {
    fn declspec(&mut self) -> Type {
        if self.consume("int") {
            return new_int();
        } else if self.consume("char") {
            return new_char();
        }
        self.error_tok(&self.tokens[0], "int or char expected");
    }

    // return (Type, name, is_function)
    // グローバル変数か、関数かの判定に使う。parseのはじめ以外では使用しない
    fn decltype(&mut self, ty: Type) -> (Type, String, bool) {
        let mut ty = ty;
        let is_func: bool;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        let name = self.get_ident();
        (ty, is_func) = self.type_suffix(ty);
        return (ty, name, is_func);
    }

    // 今は配列のみ
    // return (Type, is_function)
    fn type_suffix(&mut self, ty: Type) -> (Type, bool) {
        if self.equal("[") {
            self.advance_one_tok();
            let size = self.get_and_skip_number();
            self.skip("]");
            let (ty, _) = self.type_suffix(ty);
            return (new_array_ty(ty, size as usize), false);
        }
        if self.equal("(") {
            return (ty, true);
        }
        (ty, false)
    }

    fn declaration(&mut self) -> Node {
        let base_ty = self.declspec();
        let mut body = Vec::new();
        while !self.equal(";") {
            let (ty, name, _) = self.decltype(base_ty.clone());
            let mut node = self.create_lvar(name.clone().as_str(), ty, false);
            if self.equal("=") {
                self.advance_one_tok();
                node = new_assign(node, self.expr());
            }
            let node = new_expr_stmt(node);
            body.push(node);
            if self.equal(",") {
                self.advance_one_tok();
                continue;
            }
        }
        let node = new_block(body);
        self.skip(";");
        return node;
    }

    fn stmt(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkKeyword { name } if name == "return" => {
                self.advance_one_tok();
                let node = new_return(self.expr());
                self.skip(";");
                return node;
            }
            TokenKind::TkKeyword { name } if name == "if" => {
                self.advance_one_tok();
                let node: Node;
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let then = self.stmt();
                let mut els = None;
                if self.equal("else") {
                    self.advance_one_tok();
                    els = Some(self.stmt());
                }
                node = new_if(cond, then, els);
                return node;
            }
            TokenKind::TkKeyword { name } if name == "for" => {
                self.advance_one_tok();
                self.skip("(");
                let init = self.expr_stmt();
                let mut cond = None;
                let mut inc = None;
                if !self.equal(";") {
                    cond = Some(self.expr());
                }
                self.skip(";");
                if !self.equal(")") {
                    inc = Some(self.expr());
                }
                self.skip(")");
                let body = self.stmt();
                let node = new_for(init, cond, inc, body);
                return node;
            }
            TokenKind::TkKeyword { name } if name == "while" => {
                self.advance_one_tok();
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let body = self.stmt();
                let node = new_while(cond, body);
                return node;
            }
            TokenKind::TkPunct { str } if str == "{" => {
                self.skip("{");
                let node = self.compound_stmt();
                return node;
            }
            _ => {}
        }

        return self.expr_stmt();
    }

    fn is_typename(&mut self) -> bool {
        match &self.tokens[0].kind {
            TokenKind::TkKeyword { name } => match name.as_str() {
                "int" | "char" => return true,
                _ => return false,
            },
            _ => return false,
        }
    }

    fn compound_stmt(&mut self) -> Node {
        let mut body = Vec::new();
        while !self.consume("}") {
            if self.is_typename() {
                let mut node = self.declaration();
                add_type(&mut node);
                body.push(node);
            } else {
                let mut stmt = self.stmt();
                add_type(&mut stmt);
                body.push(stmt);
            }
        }
        let node = new_block(body);
        return node;
    }

    fn expr_stmt(&mut self) -> Node {
        if self.equal(";") {
            self.advance_one_tok();
            return null_stmt();
        }
        let node = new_expr_stmt(self.expr());
        self.skip(";");
        return node;
    }

    fn expr(&mut self) -> Node {
        return self.assign();
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "=" => {
                    self.advance_one_tok();
                    node = new_assign(node, self.assign());
                }
                _ => break,
            }
        }
        return node;
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "==" => {
                    self.advance_one_tok();
                    node = new_eq(node, self.relational());
                }
                TokenKind::TkPunct { str } if str == "!=" => {
                    self.advance_one_tok();
                    node = new_ne(node, self.relational());
                }
                _ => break,
            }
        }
        return node;
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "<" => {
                    self.advance_one_tok();
                    node = new_lt(node, self.add());
                }
                TokenKind::TkPunct { str } if str == "<=" => {
                    self.advance_one_tok();
                    node = new_le(node, self.add());
                }
                TokenKind::TkPunct { str } if str == ">" => {
                    self.advance_one_tok();
                    node = new_gt(node, self.add());
                }
                TokenKind::TkPunct { str } if str == ">=" => {
                    self.advance_one_tok();
                    node = new_ge(node, self.add());
                }
                _ => break,
            }
        }
        return node;
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "+" => {
                    self.advance_one_tok();
                    let rhs = self.mul();
                    node = self.new_add(node, rhs);
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_one_tok();
                    let mut rhs = self.mul();
                    add_type(&mut node);
                    add_type(&mut rhs);
                    // num - num
                    if is_integer_node(&node) && is_integer_node(&rhs) {
                        node = new_sub(node, rhs);
                        continue;
                    }
                    // ptr - num
                    if is_pointer_node(&node) && is_integer_node(&rhs) {
                        let r = new_mul(rhs, new_num(get_pointer_or_array_size(&node) as isize));
                        node = new_sub(node, r);
                        continue;
                    }
                    // ptr - ptr, which returns how many elements are between the two pointers
                    if is_pointer_node(&node) && is_pointer_node(&rhs) {
                        let l = Node {
                            kind: NodeKind::NdSub {
                                lhs: Box::new(node.clone()),
                                rhs: Box::new(rhs),
                            },
                            ty: Some(new_int()),
                        };
                        node = new_div(l, new_num(get_pointer_or_array_size(&node) as isize));
                        continue;
                    }
                    self.error_tok(&self.tokens[0], "invalid operands"); // ここの引数は正しいのか？
                }
                _ => break,
            }
        }
        node
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "*" => {
                    self.advance_one_tok();
                    node = new_mul(node, self.unary());
                }
                TokenKind::TkPunct { str } if str == "/" => {
                    self.advance_one_tok();
                    return new_div(node, self.unary());
                }
                _ => break,
            }
        }
        return node;
    }

    fn unary(&mut self) -> Node {
        if self.equal("+") {
            self.advance_one_tok();
            return self.unary();
        }
        if self.equal("-") {
            self.advance_one_tok();
            return new_neg(self.unary());
        }
        if self.equal("&") {
            self.advance_one_tok();
            return new_addr(self.unary());
        }
        if self.equal("*") {
            self.advance_one_tok();
            return new_deref(self.unary());
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Node {
        let mut node = self.primary();
        while self.equal("[") {
            self.advance_one_tok();
            let idx = self.expr();
            self.skip("]");
            node = new_deref(self.new_add(node, idx));
        }
        add_type(&mut node);
        return node;
    }

    // ここ、関数のtyも返すようにしたいが、自分で定義したものだけで、includeしたものをどうするかわからん
    fn funccall(&mut self, name: &str) -> Node {
        self.advance_one_tok();
        let mut args = Vec::new();
        while !self.equal(")") {
            if self.equal(",") {
                self.advance_one_tok();
            }
            args.push(self.assign());
        }
        if args.len() > 8 {
            self.error_tok(&self.tokens[0], "too many arguments");
        }
        let node = Node {
            kind: NodeKind::NdFuncCall {
                name: name.to_string(),
                args: args,
            },
            ty: Some(new_int()), // 自分で定義するようになったら、また変数リストから、型を取り出して入れる。includeの場合はどうする？足し算とかできないよな。まあ後で考えるか
        };
        self.skip(")");
        return node;
    }

    fn primary(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkNum { .. } => {
                return new_num(self.get_and_skip_number());
            }
            TokenKind::TkPunct { str } if str == "(" => {
                self.advance_one_tok();
                let node = self.expr();
                self.skip(")");
                return node;
            }
            TokenKind::TkKeyword { name } if name == "sizeof" => {
                self.advance_one_tok();
                let mut node = self.unary();
                add_type(&mut node);
                return new_num(node.ty.as_ref().unwrap().size as isize);
            }
            TokenKind::TkStr { str } => {
                let name = format!("lC{}", self.global_variables.len());
                let var = self.create_gvar(
                    name.as_str(),
                    new_array_ty(new_char(), str.len()),
                    Some(InitGval::Str(str.clone())),
                );
                self.advance_one_tok();
                return var;
            }
            TokenKind::TkIdent { name } => {
                let name = name.clone();
                self.advance_one_tok();
                let node: Node;
                // funccall
                if self.equal("(") {
                    node = self.funccall(&name);
                    return node;
                }

                // variable
                if let Some(var) = self.find_var(&name) {
                    node = Node {
                        kind: NodeKind::NdVar { var: var.clone() }, // Clone the Rc to increase the reference count
                        ty: Some(var.borrow().clone().ty),
                    };
                } else {
                    self.error_tok(&self.tokens[0], "undefined variable");
                }
                return node;
            }
            _ => self.error_tok(&self.tokens[0], "expected a number or ( expression )"),
        }
    }

    fn create_gvar(&mut self, name: &str, ty: Type, init_gval: Option<InitGval>) -> Node {
        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: self.global_variables.len() as isize, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            ty: ty.clone(),
            is_def_arg: false,
            is_local: false,
            init_gval: init_gval,
        }));

        self.global_variables.push(var.clone());
        let node = new_var(var);
        return node;
    }

    // 関数定義用の
    fn create_lvar(&mut self, name: &str, ty: Type, is_def_arg: bool) -> Node {
        let variables = &mut self
            .functions
            .get_mut(&self.processing_funcname)
            .unwrap()
            .variables;

        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: 0, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            // ここcodegenで改めてoffsetを割り当てているから意味ない説。グローバル変数はまた別で使えるかも。名前とか
            ty: ty.clone(),
            is_def_arg: is_def_arg,
            is_local: true,
            init_gval: None,
        }));

        // if !self.is_processing_local {
        //     var.borrow_mut().offset = self.global_variables.len() as isize;
        //     self.global_variables.push(var.clone());
        //     return new_var(var);
        // }
        variables.push(var.clone());
        let node = new_var(var);

        // 関数定義の引数の場合、関数のargsにも追加
        if is_def_arg {
            let func = self.functions.get_mut(&self.processing_funcname).unwrap();
            if func.args.len() >= 8 {
                self.error_tok(&self.tokens[0], "too many arguments");
            }
            func.args.push(node.clone());
        }
        return node;
    }

    fn get_ident(&mut self) -> String {
        let n: String;
        if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            n = name.clone();
        } else {
            self.error_tok(&self.tokens[0], "expected identifier");
        }
        self.advance_one_tok();
        return n;
    }

    fn new_add(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut lhs = lhs;
        let mut rhs = rhs;
        // self.advance_one_tok();
        // let mut rhs = self.mul();
        add_type(&mut lhs);
        add_type(&mut rhs);
        // num + num
        if is_integer_node(&lhs) && is_integer_node(&rhs) {
            let node = Node {
                kind: NodeKind::NdAdd {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ty: Some(new_int()),
            };
            return node;
        }
        if is_pointer_node(&lhs) && is_pointer_node(&rhs) {
            self.error_tok(&self.tokens[0], "invalid operands");
        }
        // canonicalize num + ptr -> ptr + num
        if is_integer_node(&lhs) && is_pointer_node(&rhs) {
            swap(&mut lhs, &mut rhs);
        }
        // ptr + num
        if is_pointer_node(&lhs) && is_integer_node(&rhs) {
            // node.tyのkindのptr_toのsizeを取得してvalに足す
            let size = get_pointer_or_array_size(&lhs);
            let r = new_mul(rhs, new_num(size as isize));
            let node = Node {
                kind: NodeKind::NdAdd {
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(r),
                },
                ty: Some(lhs.ty.clone().unwrap()),
            };
            return node;
        }
        return lhs;
    }

    // 今は関数だけ
    // functionとかに切り出すべき
    pub fn parse(&mut self) {
        self.tokens = self.tokenize();
        self.convert_keywords();

        while !self.tokens.is_empty() {
            self.is_processing_local = false;
            let base_ty = self.declspec();
            let (ty, name, is_func) = self.decltype(base_ty.clone());
            // 関数の場合
            if is_func {
                let variables: Vec<Rc<RefCell<Var>>> = Vec::new();
                // set processing funcname environment variable
                self.processing_funcname = name.clone();
                let func = Function {
                    name: name.clone(),
                    variables,
                    args: Vec::new(),
                    body: None,
                    ty: ty,
                };
                self.functions.insert(name.clone(), func);

                // 引数の処理
                self.skip("(");
                while !self.equal(")") {
                    let base_ty = self.declspec();
                    let (ty, name, _) = self.decltype(base_ty);
                    let mut node = self.create_lvar(name.as_str(), ty, true);
                    add_type(&mut node);
                    if self.equal(",") {
                        self.advance_one_tok();
                    }
                }
                self.skip(")");
                self.skip("{");
                self.is_processing_local = true;
                let mut node = self.compound_stmt();
                self.is_processing_local = false;
                add_type(&mut node);
                if let Some(func) = self.functions.get_mut(&name) {
                    func.body = Some(node);
                } else {
                    panic!("Function not found");
                }
            } else {
                // グローバル変数の場合
                // 今は初期化のみ
                self.create_gvar(name.as_str(), ty, None);
                while self.consume(",") {
                    let (ty, name, _) = self.decltype(base_ty.clone()); // なぜclone
                    self.create_gvar(name.as_str(), ty, None);
                }
                self.skip(";");
                continue;
            }
        }
    }
}

impl Ctx<'_> {
    pub fn find_var(&mut self, name: &str) -> Option<Rc<RefCell<Var>>> {
        let variables = &mut self
            .functions
            .get_mut(&self.processing_funcname)
            .unwrap()
            .variables
            .iter();
        for var in variables {
            if var.borrow().name == name {
                return Some(var.clone());
            }
        }
        for var in &self.global_variables {
            if var.borrow_mut().name == name {
                // なぜborrow_mut()なのか
                return Some(var.clone());
            }
        }
        None
    }
}

fn get_pointer_or_array_size(node: &Node) -> usize {
    match &node.ty {
        Some(ty) => match &ty.kind {
            TypeKind::TyPtr { ptr_to } | TypeKind::TyArray { ptr_to, .. } => ptr_to.size,
            _ => panic!("not a pointer or array"),
        },
        None => panic!("no type information"),
    }
}
