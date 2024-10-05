use crate::types::*;

impl Ctx<'_> {
    fn stmt(&mut self) -> Node {
        return self.expr_stmt();
    }
    fn expr_stmt(&mut self) -> Node {
        let node = Node {
            kind: NodeKind::NdExprStmt {
                lhs: Box::new(self.expr()),
            },
        };
        self.skip(";");
        return node;
    }
    fn expr(&mut self) -> Node {
        return self.equality();
    }
    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "==" => {
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdEq {
                            lhs: Box::new(node),
                            rhs: Box::new(self.relational()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "!=" => {
                    self.advance_tok(1);
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
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdLt {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "<=" => {
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdLe {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == ">" => {
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdGt {
                            lhs: Box::new(node),
                            rhs: Box::new(self.add()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == ">=" => {
                    self.advance_tok(1);
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
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdAdd {
                            lhs: Box::new(node),
                            rhs: Box::new(self.mul()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_tok(1);
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
                    self.advance_tok(1);
                    node = Node {
                        kind: NodeKind::NdMul {
                            lhs: Box::new(node),
                            rhs: Box::new(self.unary()),
                        },
                    };
                }
                TokenKind::TkPunct { str } if str == "/" => {
                    self.advance_tok(1);
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
            self.advance_tok(1);
            return self.unary();
        }
        if self.equal("-") {
            self.advance_tok(1);
            return Node {
                kind: NodeKind::NdNeg {
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
                self.advance_tok(1);
                let node = self.expr();
                self.skip(")");
                return node;
            }
            _ => self.error_tok(&self.tokens[0], "expected a number or ( expression )"),
        }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut program = Vec::new();
        self.tokens = self.tokenize();
        while !self.tokens.is_empty() {
            program.push(self.stmt());
        }
        return program;
    }
}
