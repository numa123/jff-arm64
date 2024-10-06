use std::{cell::RefCell, mem::swap, rc::Rc};

use crate::types::*;

impl Ctx<'_> {
    fn declspec(&mut self) -> Type {
        self.skip("int");
        return new_int();
    }
    fn decltype(&mut self, ty: Type) -> Type {
        let mut ty = ty;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        return ty;
    }
    fn declaration(&mut self) -> Node {
        let base_ty = self.declspec();
        let mut body = Vec::new();
        while !self.equal(";") {
            let ty = self.decltype(base_ty.clone());
            if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
                // eprintln!("{}\n{:#?}", name, ty);
                let mut node = self.create_var(name.clone().as_str(), ty);
                self.advance_one_tok();
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
            } else {
                // tokenがidentifierでない場合
                self.error_tok(&self.tokens[0], "expected an identifier");
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
                        let r = Node {
                            kind: NodeKind::NdMul {
                                lhs: Box::new(rhs),
                                rhs: Box::new(Node {
                                    kind: NodeKind::NdNum { val: 8 },
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
                                    kind: NodeKind::NdNum { val: 8 },
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
                                    kind: NodeKind::NdNum { val: 8 },
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

    fn create_var(&mut self, name: &str, ty: Type) -> Node {
        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: self.variables.len() as isize * 8, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            ty: ty.clone(),
        }));
        self.variables.push(var.clone());
        let node = Node {
            kind: NodeKind::NdVar { var: var.clone() },
            ty: Some(ty),
        };
        return node;
    }

    // -> Function
    pub fn parse(&mut self) {
        let mut program = Vec::new();
        self.tokens = self.tokenize();
        self.convert_keywords();
        while !self.tokens.is_empty() {
            program.push(self.stmt());
        }
        self.body = program;
    }
}

impl Ctx<'_> {
    pub fn find_var(&self, name: &str) -> Option<Rc<RefCell<Var>>> {
        for var in &self.variables {
            if var.borrow().name == name {
                return Some(var.clone());
            }
        }
        None
    }
}
