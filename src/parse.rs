use std::{cell::RefCell, rc::Rc};

use crate::types::*;

impl Ctx<'_> {
    fn stmt(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkKeyword { name } if name == "return" => {
                self.advance_one_tok();
                let node = Node {
                    kind: NodeKind::NdReturn {
                        lhs: Box::new(self.expr()),
                    },
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
            body.push(self.stmt());
        }
        let node = Node {
            kind: NodeKind::NdBlock { body },
        };
        return node;
    }
    fn expr_stmt(&mut self) -> Node {
        if self.equal(";") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdBlock { body: Vec::new() },
            };
        }
        let node = Node {
            kind: NodeKind::NdExprStmt {
                lhs: Box::new(self.expr()),
            },
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
                    };
                }
                TokenKind::TkPunct { str } if str == "!=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdNe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.relational()),
                        },
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
                    };
                }
                TokenKind::TkPunct { str } if str == "<=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdLe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == ">" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdGt {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == ">=" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdGe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
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
                    node = Node {
                        kind: NodeKind::NdAdd {
                            lhs: Box::new(node),
                            rhs: Box::new(self.mul()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdSub {
                            lhs: Box::new(node),
                            rhs: Box::new(self.mul()),
                        },
                    };
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
                    };
                }
                TokenKind::TkPunct { str } if str == "/" => {
                    self.advance_one_tok();
                    node = Node {
                        kind: NodeKind::NdDiv {
                            lhs: Box::new(node),
                            rhs: Box::new(self.unary()),
                        },
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
            };
        }
        if self.equal("&") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdAddr {
                    lhs: Box::new(self.unary()),
                },
            };
        }
        if self.equal("*") {
            self.advance_one_tok();
            return Node {
                kind: NodeKind::NdDeref {
                    lhs: Box::new(self.unary()),
                },
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
                    };
                } else {
                    let var = Rc::new(RefCell::new(Var {
                        name: name.clone(),
                        offset: self.variables.len() as isize * 8, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
                    }));
                    self.variables.push(var.clone());
                    node = Node {
                        kind: NodeKind::NdVar { var: var.clone() },
                    };
                }
                return node;
            }
            _ => self.error_tok(&self.tokens[0], "expected a number or ( expression )"),
        }
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
