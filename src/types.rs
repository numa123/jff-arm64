#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    TkPunct,
    TkNum,
    TkIdent, // variable
    TkKeyword,
    TkStr,
}

#[derive(Debug, Clone, PartialEq)]
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
    NdMod,      // %
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
    NdAddr,     // &
    NdDeref,    // *
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub ty: Type,
    pub offset: usize,
    pub def_arg: bool, // true if this variable is a function argument
    // pub is_local: bool,
    // only for function
    pub is_func: bool,
    pub stmts: Vec<Node>,    // stmts(?)
    pub variables: Vec<Var>, // variables including function arguments
    pub args: Vec<Node>,     // only function arguments
    pub gval: Option<i32>,   // global variable value
    pub str: Option<String>, // string literal
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Type>,
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
    // tokをつけて、error_tokを使いたい
}

// #[derive(Debug, Clone)]
// pub struct Function {
//     pub name: String,
//     pub ty: Type,
//     pub stmts: Vec<Node>,    // stmts(?)
//     pub variables: Vec<Var>, // variables including function arguments
//     pub args: Vec<Node>,     // only function arguments
// }

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    TyInt,
    TyPtr,
    TyArray,
    TyChar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub size: usize,
    pub ptr_to: Option<Box<Type>>,
    pub tok: Option<Token>,
    pub array_len: Option<usize>,
}

//
//
//
pub fn is_integer(ty: &Type) -> bool {
    match ty.kind {
        TypeKind::TyInt | TypeKind::TyChar => true,
        _ => false,
    }
}

pub fn is_pointer(ty: &Type) -> bool {
    match ty.kind {
        TypeKind::TyPtr => true,
        _ => false,
    }
}

pub fn new_ptr_to(ty: Type) -> Type {
    Type {
        kind: TypeKind::TyPtr,
        size: 8,
        ptr_to: Some(Box::new(ty)),
        tok: None,
        array_len: None,
    }
}

pub fn new_int() -> Type {
    Type {
        kind: TypeKind::TyInt,
        size: 8,
        ptr_to: None,
        tok: None,
        array_len: None,
    }
}

pub fn new_array(ty: Type, len: usize) -> Type {
    Type {
        kind: TypeKind::TyArray,
        size: ty.size * len, // まだ要素がポインタか、8byteの整数のみしかないので、8byte固定
        ptr_to: Some(Box::new(ty)),
        tok: None,
        array_len: Some(len),
    }
}

pub fn new_char() -> Type {
    Type {
        kind: TypeKind::TyChar,
        size: 1,
        ptr_to: None,
        tok: None,
        array_len: None,
    }
}

pub fn is_typename(token: Token) -> bool {
    match token.str.as_str() {
        "int" | "char" => true,
        _ => false,
    }
}

pub fn add_type(node: &mut Node) {
    if node.ty.is_some() {
        return;
    }

    if node.lhs.is_some() {
        add_type(node.lhs.as_mut().unwrap());
    }
    if node.rhs.is_some() {
        add_type(node.rhs.as_mut().unwrap());
    }
    if node.cond.is_some() {
        add_type(node.cond.as_mut().unwrap());
    }
    if node.then.is_some() {
        add_type(node.then.as_mut().unwrap());
    }
    if node.els.is_some() {
        add_type(node.els.as_mut().unwrap());
    }
    if node.init.is_some() {
        add_type(node.init.as_mut().unwrap());
    }
    if node.inc.is_some() {
        add_type(node.inc.as_mut().unwrap());
    }

    for body in node.block_body.iter_mut() {
        add_type(body);
    }

    for arg in node.args.iter_mut() {
        add_type(arg);
    }

    match node.kind {
        NodeKind::NdAdd | NodeKind::NdSub | NodeKind::NdMul | NodeKind::NdDiv | NodeKind::NdNeg => {
            node.ty = Some(node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone());
        }
        NodeKind::NdAssign => {
            if node.lhs.as_ref().unwrap().clone().ty.unwrap().kind == TypeKind::TyArray {
                panic!("add_type: not an lvalue");
            }
            node.ty = Some(node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone());
        }
        NodeKind::NdEq
        | NodeKind::NdNe
        | NodeKind::NdLt
        | NodeKind::NdLe
        | NodeKind::NdGt
        | NodeKind::NdGe
        | NodeKind::NdNum => {
            node.ty = Some(new_int());
        }
        NodeKind::NdVar => {
            node.ty = Some(node.var.as_ref().unwrap().ty.clone()); // この辺わからん、as_refとか
        }
        NodeKind::NdAddr => {
            node.ty = Some(new_ptr_to(
                node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone(),
            ));
        }
        NodeKind::NdDeref => {
            if node
                .lhs
                .as_ref()
                .unwrap()
                .ty
                .as_ref()
                .unwrap()
                .ptr_to
                .is_none()
            {
                panic!("invalid pointer dereference");
            }
            node.ty = Some(new_ptr_to(
                node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone(),
            ));
        }
        NodeKind::NdFuncCall => {
            node.ty = Some(new_int()); // int以外の場合もあるでしょ、いや、いいのか？それは受け止める側がやればよいのか
        }
        _ => {}
    }
}
