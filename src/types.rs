#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    TkPunct,
    TkNum,
    TkIdent, // variable
    TkKeyword,
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
    pub offset: usize,
    pub def_arg: bool, // true if this variable is a function argument
                       // pub ty: Type,
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
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub stmts: Vec<Node>,    // stmts(?)
    pub variables: Vec<Var>, // variables including function arguments
    pub args: Vec<Node>,     // only function arguments
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    TyInt,
    TyPtr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
    // pub name: Option<Token>, // nameがTokenってどういうことだ。一意性を保証するためかな。nameは不適切では？ // declaration
}

//
//
//
pub fn is_integer(ty: &Type) -> bool {
    match ty.kind {
        TypeKind::TyInt => true,
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
        ptr_to: Some(Box::new(ty)),
        // name: None,
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
        NodeKind::NdAdd
        | NodeKind::NdSub
        | NodeKind::NdMul
        | NodeKind::NdDiv
        | NodeKind::NdNeg
        | NodeKind::NdAssign => {
            node.ty = Some(node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone());
        }
        NodeKind::NdEq
        | NodeKind::NdNe
        | NodeKind::NdLt
        | NodeKind::NdLe
        | NodeKind::NdGt
        | NodeKind::NdGe
        | NodeKind::NdNum
        | NodeKind::NdVar => {
            node.ty = Some(Type {
                kind: TypeKind::TyInt,
                ptr_to: None,
                // name: None,
            });
        }
        NodeKind::NdAddr => {
            node.ty = Some(new_ptr_to(
                node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone(),
            ));
        }
        NodeKind::NdDeref => {
            if node.lhs.as_ref().unwrap().ty.as_ref().unwrap().kind == TypeKind::TyInt {
                node.ty = Some(Type {
                    kind: TypeKind::TyInt,
                    ptr_to: None,
                    // name: None,
                });
            } else {
                node.ty = Some(new_ptr_to(
                    node.lhs.as_ref().unwrap().ty.as_ref().unwrap().clone(),
                ));
            }
        }
        NodeKind::NdFuncCall => {
            node.ty = Some(Type {
                kind: TypeKind::TyInt,
                ptr_to: None,
                // name: None,
            });
        }
        _ => {}
    }
}
