use std::{cell::RefCell, collections::HashMap, rc::Rc};

// いずれ
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub variables: Vec<Rc<RefCell<Var>>>,
    pub body: Option<Node>, // {compound_stmt}
    pub args: Vec<Node>,    // Vec<Rc<RefCell<Var>>>にするかも。可変長引数の場合。
    pub ty: Type,
}
#[derive(Debug)]
pub struct Ctx<'a> {
    pub input: &'a str,
    pub input_copy: &'a str,
    pub tokens: Vec<Token>,
    // global_variables: Vec<Var>, いずれ
    pub processing_funcname: String,
    pub functions: HashMap<String, Function>,
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
    pub ty: Type,
    pub is_def_arg: bool,
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
    NdFuncCall {
        name: String,
        args: Vec<Node>,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    TyInt,
    TyPtr { ptr_to: Box<Type> },
    TyArray { ptr_to: Box<Type>, len: usize },
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub size: usize,
}

pub fn new_ptr_to(ty: Type) -> Type {
    Type {
        kind: TypeKind::TyPtr {
            ptr_to: Box::new(ty),
        },
        size: 8,
    }
}

pub fn new_int() -> Type {
    Type {
        kind: TypeKind::TyInt,
        size: 8, // long?
    }
}

pub fn new_array_ty(ty: Type, len: usize) -> Type {
    Type {
        kind: TypeKind::TyArray {
            ptr_to: Box::new(ty.clone()),
            len,
        },
        size: ty.size * len,
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
    match &node.ty {
        Some(ty) => match &ty.kind {
            TypeKind::TyPtr { .. } | TypeKind::TyArray { .. } => true,
            _ => false,
        },
        None => false,
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
    if let TypeKind::TyPtr { .. } = ty.kind {
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
                match &ty.kind {
                    TypeKind::TyPtr { ptr_to } | TypeKind::TyArray { ptr_to, .. } => {
                        node.ty = Some((**ptr_to).clone());
                    }
                    _ => panic!("not a pointer or array"),
                }
            } else {
                panic!("no type information");
            }
        }
        NodeKind::NdReturn { lhs } | NodeKind::NdExprStmt { lhs } | NodeKind::NdNeg { lhs } => {
            add_type(lhs);
            node.ty = lhs.ty.clone();
        }
        _ => {} // Block, If, For, While
    }
}
