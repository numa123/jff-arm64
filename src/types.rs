#[derive(Debug, PartialEq)]
pub enum TokenKind {
    TkPunct,
    TkNum,
    TkIdent, // variable
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub val: i32,
    pub str: String,
    pub loc: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    NdAdd,    // +
    NdSub,    // -
    NdMul,    // *
    NdDiv,    // /
    NdNum,    // number
    NdNeg,    // unary =
    NdEq,     // ==
    NdNe,     // !=
    NdLt,     // <
    NdLe,     // <=
    NdGt,     // >
    NdGe,     // >=
    NdVar,    // variable
    NdAssign, // =
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i32,
    pub var: Option<Box<Var>>, // Used if kind == NdVar
}
