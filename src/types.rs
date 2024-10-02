#[derive(Debug, PartialEq)]
pub enum TokenKind {
    TkPunct,
    TkNum,
    TkIdent, // variable
    TkKeyword,
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
    NdAdd,      // +
    NdSub,      // -
    NdMul,      // *
    NdDiv,      // /
    NdNum,      // number
    NdNeg,      // unary =
    NdEq,       // ==
    NdNe,       // !=
    NdLt,       // <
    NdLe,       // <=
    NdGt,       // >
    NdGe,       // >=
    NdVar,      // variable
    NdAssign,   // =
    NdExprStmt, // expression statement
    NdReturn,   // return
    NdBlock,    // {}
    NdIf,       // if
    NdFor,      // for or while
    NdFuncCall, // function call
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
    pub block_body: Vec<Node>, // Used if kind == NdBlock
    // if, for, while
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    // only if
    pub els: Option<Box<Node>>,
    // only for, while
    pub init: Option<Box<Node>>,
    pub inc: Option<Box<Node>>,
    pub func_name: String,
    pub args: Vec<Node>,
}

pub struct Function {
    pub name: String,
    pub stmts: Vec<Node>, // stmts(?)
    pub variables: Vec<Var>,
}
