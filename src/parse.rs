use std::{cell::RefCell, mem::swap, rc::Rc};

use crate::types::*;

impl Ctx<'_> {
    fn declspec(&mut self) -> Type {
        self.skip("int");
        return new_int();
    }
    fn decltype(&mut self, ty: Type) -> (Type, String) {
        let mut ty = ty;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        let name = self.get_ident();
        ty = self.type_suffix(ty);
        return (ty, name);
    }
    // 今は配列のみ
    fn type_suffix(&mut self, ty: Type) -> Type {
        if self.equal("[") {
            self.advance_one_tok();
            let size = self.get_and_skip_number();
            self.skip("]");
            let ty = self.type_suffix(ty);
            return new_array_ty(ty, size as usize);
        }
        ty
    }

    fn declaration(&mut self) -> Node {
        let base_ty = self.declspec();
        let mut body = Vec::new();
        while !self.equal(";") {
            let (ty, name) = self.decltype(base_ty.clone());
            let mut node = self.create_lvar(name.clone().as_str(), ty, false);
            if self.equal("=") {
                self.advance_one_tok();
                node = Node {
                    kind: NodeKind::NdAssign {
                        lhs: Box::new(node),
                        rhs: Box::new(self.expr()),
                    },
                    ty: None,
                };
            }
            let mut node = Node {
                kind: NodeKind::NdExprStmt {
                    lhs: Box::new(node),
                },
                ty: None,
            };
            add_type(&mut node);
            body.push(node);
            if self.equal(",") {
                self.advance_one_tok();
                continue;
            }
        }
        let node = Node {
            kind: NodeKind::NdBlock { body },
            ty: None,
        };
        self.skip(";");
        return node;
    }
    fn stmt(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkKeyword { name } if name == "return" => {
                self.advance_one_tok();
                let node = Node {
                    kind: NodeKind::NdReturn {
                        lhs: Box::new(self.expr()),
                    },
                    ty: None,
                };
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
                node = Node {
                    kind: NodeKind::NdIf {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        els: els.map(Box::new),
                    },
                    ty: None,
                };
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
                let node = Node {
                    kind: NodeKind::NdFor {
                        init: Box::new(init),
                        cond: cond.map(Box::new),
                        inc: inc.map(Box::new),
                        body: Box::new(body),
                    },
                    ty: None,
                };
                return node;
            }
            TokenKind::TkKeyword { name } if name == "while" => {
                self.advance_one_tok();
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let body = self.stmt();
                let node = Node {
                    kind: NodeKind::NdWhile {
                        cond: Box::new(cond),
                        body: Box::new(body),
                    },
                    ty: None,
                };
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
    fn compound_stmt(&mut self) -> Node {
        let mut body = Vec::new();
        while !self.consume("}") {
            if self.equal("int") {
                let mut node = self.declaration();
                add_type(&mut node);
                body.push(node);
            } else {
                let mut stmt = self.stmt();
                add_type(&mut stmt);
                body.push(stmt);
            }
        }
        let node = Node {
            kind: NodeKind::NdBlock { body },
            ty: None,
        };
        return node;
    }
    fn expr_stmt(&mut self) -> Node {
        if self.equal(";") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdBlock { body: Vec::new() },
                ty: None,
            };
        }
        let node = Node {
            kind: NodeKind::NdExprStmt {
                lhs: Box::new(self.expr()),
            },
            ty: None,
        };
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
                    node = Node {
                        kind: NodeKind::NdAssign {
                            lhs: Box::new(node),
                            rhs: Box::new(self.assign()),
                        },
                        ty: None,
                    };
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
                    node = Node {
                        kind: NodeKind::NdEq {
                            lhs: Box::new(node),
                            rhs: Box::new(self.relational()),
                        },
                        ty: None,
                    };
                }
                TokenKind::TkPunct { str } if str == "!=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdNe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.relational()),
                        },
                        ty: None,
                    };
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
                    node = Node {
                        kind: NodeKind::NdLt {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                        ty: None,
                    };
                }
                TokenKind::TkPunct { str } if str == "<=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdLe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                        ty: None,
                    };
                }
                TokenKind::TkPunct { str } if str == ">" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdGt {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                        ty: None,
                    };
                }
                TokenKind::TkPunct { str } if str == ">=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdGe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                        ty: None,
                    };
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
                    let mut rhs = self.mul();
                    add_type(&mut node);
                    add_type(&mut rhs);
                    // num + num
                    if is_integer_node(&node) && is_integer_node(&rhs) {
                        node = Node {
                            kind: NodeKind::NdAdd {
                                lhs: Box::new(node.clone()), // cloneか...
                                rhs: Box::new(rhs),
                            },
                            ty: node.ty,
                        };
                        continue;
                    }
                    if is_pointer_node(&node) && is_pointer_node(&rhs) {
                        self.error_tok(&self.tokens[0], "invalid operands");
                    }
                    // canonicalize num + ptr -> ptr + num
                    if is_integer_node(&node) && is_pointer_node(&rhs) {
                        swap(&mut node, &mut rhs);
                    }
                    // ptr + num
                    if is_pointer_node(&node) && is_integer_node(&rhs) {
                        // node.tyのkindのptr_toのsizeを取得してvalに足す
                        let size = get_pointer_or_array_size(&node);
                        // eprintln!("size: {}", size);
                        let r = Node {
                            kind: NodeKind::NdMul {
                                lhs: Box::new(rhs),
                                rhs: Box::new(Node {
                                    kind: NodeKind::NdNum { val: size as isize }, // usizeの限界を超えたらエラーになりそう
                                    ty: Some(new_int()),
                                }),
                            },
                            ty: None,
                        };
                        node = Node {
                            kind: NodeKind::NdAdd {
                                lhs: Box::new(node.clone()),
                                rhs: Box::new(r),
                            },
                            ty: node.ty,
                        };
                        // eprintln!("{:#?}", node);
                        continue;
                    }
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_one_tok();
                    let mut rhs = self.mul();
                    add_type(&mut node);
                    add_type(&mut rhs);
                    // num - num
                    if is_integer_node(&node) && is_integer_node(&rhs) {
                        node = Node {
                            kind: NodeKind::NdSub {
                                lhs: Box::new(node.clone()), // cloneか...
                                rhs: Box::new(rhs),
                            },
                            ty: node.ty,
                        };
                        continue;
                    }
                    // ptr - num
                    if is_pointer_node(&node) && is_integer_node(&rhs) {
                        let r = Node {
                            kind: NodeKind::NdMul {
                                lhs: Box::new(rhs),
                                rhs: Box::new(Node {
                                    kind: NodeKind::NdNum {
                                        val: get_pointer_or_array_size(&node) as isize,
                                    },
                                    ty: Some(new_int()),
                                }),
                            },
                            ty: None,
                        };
                        node = Node {
                            kind: NodeKind::NdSub {
                                lhs: Box::new(node.clone()),
                                rhs: Box::new(r),
                            },
                            ty: node.ty,
                        };
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
                        node = Node {
                            kind: NodeKind::NdDiv {
                                lhs: Box::new(l),
                                rhs: Box::new(Node {
                                    kind: NodeKind::NdNum {
                                        val: get_pointer_or_array_size(&node) as isize,
                                    },
                                    ty: Some(new_int()),
                                }),
                            },
                            ty: None, // あとで付与されるはず
                        };
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
                    node = Node {
                        kind: NodeKind::NdMul {
                            lhs: Box::new(node),
                            rhs: Box::new(self.unary()),
                        },
                        ty: None,
                    };
                }
                TokenKind::TkPunct { str } if str == "/" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdDiv {
                            lhs: Box::new(node),
                            rhs: Box::new(self.unary()),
                        },
                        ty: None,
                    };
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
            return Node {
                kind: NodeKind::NdNeg {
                    lhs: Box::new(self.unary()),
                },
                ty: None,
            };
        }
        if self.equal("&") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdAddr {
                    lhs: Box::new(self.unary()),
                },
                ty: None,
            };
        }
        if self.equal("*") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdDeref {
                    lhs: Box::new(self.unary()),
                },
                ty: None,
            };
        }
        self.primary()
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
                return Node {
                    kind: NodeKind::NdNum {
                        val: self.get_and_skip_number(),
                    },
                    ty: None,
                };
            }
            TokenKind::TkPunct { str } if str == "(" => {
                self.advance_one_tok();
                let node = self.expr();
                self.skip(")");
                return node;
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

    // 関数定義用の
    fn create_lvar(&mut self, name: &str, ty: Type, is_def_arg: bool) -> Node {
        let variables = &mut self
            .functions
            .get_mut(&self.processing_funcname)
            .unwrap()
            .variables;

        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: variables.len() as isize * 8, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            ty: ty.clone(),
            is_def_arg: is_def_arg,
        }));
        variables.push(var.clone());
        let node = Node {
            kind: NodeKind::NdVar { var: var.clone() },
            ty: Some(ty),
        };

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

    // 今は関数だけ
    // functionとかに切り出すべき
    pub fn parse(&mut self) {
        self.tokens = self.tokenize();
        self.convert_keywords();
        while !self.tokens.is_empty() {
            let base_ty = self.declspec();
            let (ty, name) = self.decltype(base_ty);
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
                let (ty, name) = self.decltype(base_ty);
                let mut node = self.create_lvar(name.as_str(), ty, true);
                add_type(&mut node);
                if self.equal(",") {
                    self.advance_one_tok();
                }
            }
            self.skip(")");
            self.skip("{");
            let mut node = self.compound_stmt();
            add_type(&mut node);
            if let Some(func) = self.functions.get_mut(&name) {
                func.body = Some(node);
            } else {
                panic!("Function not found");
            }
        }
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
