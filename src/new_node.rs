use std::cell::RefCell;
use std::rc::Rc;

use crate::type_utils::*;
use crate::types::*;

impl Ctx<'_> {
    pub fn create_func(&mut self, name: &str, ty: Type) -> Function {
        Function {
            name: name.to_string(),
            args: Vec::new(),
            body: None,
            ty,
            scopes: Vec::new(),
            scope_idx: -1, // 最初のスコープは-1にすることで、enter_scopeで良い感じに辻褄合わせ。でも、普通にわかりづらいから後で直す
            exited_scope: Vec::new(),
            is_def: true,
        }
    }

    pub fn new_if(&mut self, cond: Node, then: Node, els: Option<Node>) -> Node {
        let mut node = Node {
            kind: NodeKind::If {
                cond: Box::new(cond),
                then: Box::new(then),
                els: els.map(Box::new),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_for(
        &mut self,
        init: Node,
        cond: Option<Node>,
        inc: Option<Node>,
        body: Node,
    ) -> Node {
        let mut node = Node {
            kind: NodeKind::For {
                init: Box::new(init),
                cond: cond.map(Box::new),
                inc: inc.map(Box::new),
                body: Box::new(body),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_while(&mut self, cond: Node, body: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::While {
                cond: Box::new(cond),
                body: Box::new(body),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_block(&mut self, body: Vec<Node>) -> Node {
        let mut node = Node {
            kind: NodeKind::Block { body },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn null_stmt(&self) -> Node {
        Node {
            kind: NodeKind::Block { body: Vec::new() },
            ty: None,
        }
    }

    pub fn new_expr_stmt(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::ExprStmt { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_assign(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdAssign {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_return(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Return { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_eq(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Eq {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_ne(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Ne {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_lt(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Lt {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_le(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Le {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_gt(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Gt {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_ge(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Ge {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_and(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::And {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_or(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Or {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_neg(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Neg { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_addr(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Addr { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_deref(&mut self, lhs: Node, tok: Token) -> Node {
        let mut node = Node {
            kind: NodeKind::Deref {
                lhs: Box::new(lhs),
                tok,
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_mul(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Mul {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_div(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Div {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_mod(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::Mod {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_num(&mut self, val: isize) -> Node {
        let mut node = Node {
            kind: NodeKind::Num { val },
            ty: Some(new_int_ty()),
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_long(&mut self, val: isize) -> Node {
        Node {
            kind: NodeKind::Num { val },
            ty: Some(new_long_ty()),
        }
    }

    pub fn new_var(&mut self, var: Rc<RefCell<Var>>) -> Node {
        let mut node = Node {
            kind: NodeKind::Var { var },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_bit_and(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::BitAnd {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_bit_xor(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::BitXor {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_bit_or(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::BitOr {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        node
    }

    pub fn new_cast(&mut self, lhs: Node, ty: Type) -> Node {
        let mut node = Node {
            kind: NodeKind::Cast { lhs: Box::new(lhs) },
            ty: Some(ty),
        };
        self.add_type(&mut node);
        node
    }
}
