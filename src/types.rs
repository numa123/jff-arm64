use std::{cell::RefCell, rc::Rc};

// いずれ
// #[derive(Debug)]
// pub struct Function {
//     pub variables: Vec<Var>,
//     pub body: Vec<Node>, // 今はstmt*
//                          // stack_size: usize,
// }
#[derive(Debug)]
pub struct Ctx<'a> {
    pub input: &'a str,
    pub input_copy: &'a str,
    pub tokens: Vec<Token>,
    pub variables: Vec<Rc<RefCell<Var>>>,
    pub body: Vec<Node>, // 今はstmt*
                         // pub functions: Vec<Function>, いずれ
                         // global_variables: Vec<Var>, いずれ
}

#[derive(Debug)]
pub enum TokenKind {
    TkPunct { str: String },
    TkNum { val: isize },
    TkIdent { name: String },
    TkReturn,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub offset: isize,
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
    NdExprStmt { lhs: Box<Node> },
    NdNum { val: isize },
    NdAssign { lhs: Box<Node>, rhs: Box<Node> },
    NdVar { var: Rc<RefCell<Var>> },
    NdReturn { lhs: Box<Node> },
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
}
