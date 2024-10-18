use core::panic;
use std::{cell::RefCell, rc::Rc};

use crate::types::*;

pub fn new_ptr_to(ty: Type) -> Type {
    Type {
        kind: TypeKind::Ptr {
            ptr_to: Box::new(ty),
        },
        size: 8,
        align: 8,
    }
}

pub fn new_short() -> Type {
    Type {
        kind: TypeKind::Short,
        size: 2,
        align: 2,
    }
}

pub fn new_int_ty() -> Type {
    Type {
        kind: TypeKind::Int,
        size: 4,
        align: 4,
    }
}

pub fn new_long_ty() -> Type {
    Type {
        kind: TypeKind::Long,
        size: 8,
        align: 8,
    }
}

pub fn new_char_ty() -> Type {
    Type {
        kind: TypeKind::Char,
        size: 1,
        align: 1,
    }
}

pub fn new_array_ty(ty: Type, len: usize) -> Type {
    Type {
        kind: TypeKind::Array {
            ptr_to: Box::new(ty.clone()),
            len,
        },
        size: ty.size * len,
        align: ty.align,
    }
}

pub fn get_common_type(ty1: Type, ty2: Type) -> Type {
    if let TypeKind::Ptr { ptr_to } = ty1.kind {
        return new_ptr_to(*ptr_to); // こうなのか？なぜかは知らん
    }
    if ty1.size == 8 || ty2.size == 8 {
        return new_long_ty();
    }
    new_int_ty() // short + shortもintになるらしい
}

pub fn is_integer_node(node: &Node) -> bool {
    if let Some(ty) = &node.ty {
        matches!(
            ty.kind,
            TypeKind::Int
                | TypeKind::Short
                | TypeKind::Char
                | TypeKind::Long
                | TypeKind::Enum { .. }
        )
    } else {
        false
    }
}

pub fn is_pointer_node(node: &Node) -> bool {
    match &node.ty {
        Some(ty) => matches!(ty.kind, TypeKind::Ptr { .. } | TypeKind::Array { .. }),
        None => false,
    }
}

#[allow(dead_code)]
pub fn is_integer(ty: &Type) -> bool {
    matches!(ty.kind, TypeKind::Int | TypeKind::Short | TypeKind::Long)
}

#[allow(dead_code)]
pub fn is_pointer(ty: &Type) -> bool {
    matches!(ty.kind, TypeKind::Ptr { .. })
}

pub fn get_pointer_or_array_size(node: &Node) -> usize {
    match &node.ty {
        Some(ty) => match &ty.kind {
            TypeKind::Ptr { ptr_to } | TypeKind::Array { ptr_to, .. } => ptr_to.size,
            _ => panic!("not a pointer or array"),
        },
        None => panic!("no type information"),
    }
}

pub fn copy_type(node: &Node) -> Type {
    node.clone().ty.unwrap()
}

pub fn copy_var_type(var: &Rc<RefCell<Var>>) -> Type {
    var.borrow().clone().ty
}

impl Ctx<'_> {
    pub fn add_type(&mut self, node: &mut Node) {
        if node.ty.is_some() {
            return;
        }

        match &mut node.kind {
            NodeKind::Add { lhs, rhs }
            | NodeKind::Sub { lhs, rhs }
            | NodeKind::Mul { lhs, rhs }
            | NodeKind::Div { lhs, rhs }
            | NodeKind::Mod { lhs, rhs } => {
                self.add_type(lhs);
                self.add_type(rhs);
                let ty = get_common_type(copy_type(lhs), copy_type(rhs));
                node.ty = Some(ty);
            }
            NodeKind::NdAssign { lhs, rhs } => {
                self.add_type(lhs);
                self.add_type(rhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::Eq { lhs, rhs }
            | NodeKind::Ne { lhs, rhs }
            | NodeKind::Lt { lhs, rhs }
            | NodeKind::Le { lhs, rhs }
            | NodeKind::Gt { lhs, rhs }
            | NodeKind::Ge { lhs, rhs }
            | NodeKind::And { lhs, rhs }
            | NodeKind::Or { lhs, rhs } => {
                self.usual_arith_conv(lhs, rhs);
                node.ty = Some(new_int_ty());
            }
            NodeKind::Num { val } => {
                node.ty = if *val == (*val as i32 as isize) {
                    Some(new_int_ty())
                } else {
                    Some(new_long_ty())
                }
            }
            NodeKind::Var { var } => {
                node.ty = Some(var.borrow().ty.clone());
            }
            NodeKind::Addr { lhs } => {
                self.add_type(lhs);
                if let Some(ty) = &lhs.ty {
                    node.ty = Some(new_ptr_to(ty.clone()));
                }
            }
            NodeKind::Deref { lhs, tok } => {
                self.add_type(lhs);
                if let Some(ty) = &lhs.ty {
                    match &ty.kind {
                        TypeKind::Ptr { ptr_to } | TypeKind::Array { ptr_to, .. } => {
                            node.ty = Some((**ptr_to).clone());
                        }
                        _ => self.error_tok(tok, "not a pointer or array"),
                    }
                } else {
                    panic!("no type information");
                }
            }
            NodeKind::Neg { lhs } => {
                self.add_type(lhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::Return { lhs } | NodeKind::ExprStmt { lhs } => {
                self.add_type(lhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::GNUStmtExpr { body } => {
                let last_node = body.last().unwrap();
                node.ty = last_node.ty.clone();
            }
            NodeKind::Member { member, .. } => {
                node.ty = Some(member.ty.clone()); // よくわからん
            }
            _ => {} // Block, If, For, While, Funccall
        }
    }

    pub fn usual_arith_conv(&mut self, lhs: &mut Node, rhs: &mut Node) {
        let ty = get_common_type(copy_type(lhs), copy_type(lhs));
        *lhs = self.new_cast(lhs.clone(), ty.clone());
        *rhs = self.new_cast(rhs.clone(), ty);
    }
}
