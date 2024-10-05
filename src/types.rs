#[derive(Debug)]
pub struct Ctx<'a> {
    pub input: &'a str,
    pub input_copy: &'a str,
    pub tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum TokenKind {
    TkPunct { str: String },
    TkNum { val: isize },
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

#[derive(Debug)]
pub enum NodeKind {
    NdAdd { lhs: Box<Node>, rhs: Box<Node> },
    NdSub { lhs: Box<Node>, rhs: Box<Node> },
    NdMul { lhs: Box<Node>, rhs: Box<Node> },
    NdDiv { lhs: Box<Node>, rhs: Box<Node> },
    NdNeg { lhs: Box<Node> },
    NdEq { lhs: Box<Node>, rhs: Box<Node> },
    NdNe { lhs: Box<Node>, rhs: Box<Node> },
    NdLt { lhs: Box<Node>, rhs: Box<Node> },
    NdLe { lhs: Box<Node>, rhs: Box<Node> },
    NdGt { lhs: Box<Node>, rhs: Box<Node> },
    NdGe { lhs: Box<Node>, rhs: Box<Node> },
    NdNum { val: isize },
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
}
