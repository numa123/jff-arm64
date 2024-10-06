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
    TkKeyword { name: String },
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

#[derive(Debug, Clone)]
pub enum NodeKind {
    NdAdd {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdSub {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdMul {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdDiv {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdNeg {
        lhs: Box<Node>,
    },
    NdEq {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdNe {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdLt {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdLe {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdGt {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdGe {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdExprStmt {
        lhs: Box<Node>,
    },
    NdNum {
        val: isize,
    },
    NdAssign {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdVar {
        var: Rc<RefCell<Var>>,
    },
    NdReturn {
        lhs: Box<Node>,
    },
    NdBlock {
        body: Vec<Node>,
    },
    NdIf {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    },
    NdFor {
        init: Box<Node>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        body: Box<Node>,
    },
    NdWhile {
        cond: Box<Node>,
        body: Box<Node>,
    },
    NdAddr {
        lhs: Box<Node>,
    },
    NdDeref {
        lhs: Box<Node>,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
enum TypeKind {
    TyInt,
    TyPtr,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub ptr_to: Option<Box<Type>>,
}

pub fn new_ptr_to(ty: Type) -> Type {
    Type {
        kind: TypeKind::TyPtr,
        ptr_to: Some(Box::new(ty)),
    }
}

pub fn new_int() -> Type {
    Type {
        kind: TypeKind::TyInt,
        ptr_to: None,
    }
}

pub fn is_integer_node(node: &Node) -> bool {
    if let Some(ty) = &node.ty {
        if let TypeKind::TyInt = ty.kind {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn is_pointer_node(node: &Node) -> bool {
    if let Some(ty) = &node.ty {
        if let TypeKind::TyPtr = ty.kind {
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_integer(ty: &Type) -> bool {
    if let TypeKind::TyInt = ty.kind {
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_pointer(ty: &Type) -> bool {
    if let TypeKind::TyPtr = ty.kind {
        true
    } else {
        false
    }
}

pub fn add_type(node: &mut Node) {
    if node.ty.is_some() {
        return;
    }

    match &mut node.kind {
        NodeKind::NdAdd { lhs, rhs }
        | NodeKind::NdSub { lhs, rhs }
        | NodeKind::NdMul { lhs, rhs }
        | NodeKind::NdDiv { lhs, rhs }
        | NodeKind::NdAssign { lhs, rhs } => {
            add_type(lhs);
            add_type(rhs);
            node.ty = lhs.ty.clone();
        }
        NodeKind::NdEq { .. }
        | NodeKind::NdNe { .. }
        | NodeKind::NdLt { .. }
        | NodeKind::NdLe { .. }
        | NodeKind::NdGt { .. }
        | NodeKind::NdGe { .. }
        | NodeKind::NdNum { .. } => {
            node.ty = Some(new_int());
        }
        NodeKind::NdVar { .. } => {
            node.ty = Some(new_int());
        }
        NodeKind::NdAddr { lhs } => {
            add_type(lhs);
            if let Some(ty) = &lhs.ty {
                node.ty = Some(new_ptr_to(ty.clone()));
            }
        }
        NodeKind::NdDeref { lhs } => {
            add_type(lhs);
            if let Some(ty) = &lhs.ty {
                if let TypeKind::TyPtr = ty.kind {
                    node.ty = Some(*(ty.ptr_to.clone().unwrap()));
                } else {
                    node.ty = Some(new_int());
                }
            }
        }
        NodeKind::NdReturn { lhs } | NodeKind::NdExprStmt { lhs } | NodeKind::NdNeg { lhs } => {
            add_type(lhs);
            node.ty = lhs.ty.clone();
        }
        _ => {} // Block, If, For, While
    }
}
