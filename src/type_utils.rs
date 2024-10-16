use core::panic;

use crate::types::*;

pub fn new_ptr_to(ty: Type) -> Type {
    Type {
        kind: TypeKind::TyPtr {
            ptr_to: Box::new(ty),
        },
        size: 8,
        align: 8,
    }
}

pub fn new_short() -> Type {
    Type {
        kind: TypeKind::TyShort,
        size: 2,
        align: 2,
    }
}

pub fn new_int_ty() -> Type {
    Type {
        kind: TypeKind::TyInt,
        size: 4,
        align: 4,
    }
}

pub fn new_long_ty() -> Type {
    Type {
        kind: TypeKind::TyLong,
        size: 8,
        align: 8,
    }
}

pub fn new_char_ty() -> Type {
    Type {
        kind: TypeKind::TyChar,
        size: 1,
        align: 1,
    }
}

pub fn new_array_ty(ty: Type, len: usize) -> Type {
    Type {
        kind: TypeKind::TyArray {
            ptr_to: Box::new(ty.clone()),
            len,
        },
        size: ty.size * len,
        align: ty.align,
    }
}

pub fn get_common_type(ty1: Type, ty2: Type) -> Type {
    if let TypeKind::TyPtr { ptr_to } = ty1.kind {
        return new_ptr_to(*ptr_to); // こうなのか？なぜかは知らん
    }
    if ty1.size == 8 || ty2.size == 8 {
        return new_long_ty();
    }
    return new_int_ty(); // short + shortもintになるらしい
}

pub fn is_integer_node(node: &Node) -> bool {
    if let Some(ty) = &node.ty {
        match ty.kind {
            TypeKind::TyInt
            | TypeKind::TyShort
            | TypeKind::TyChar
            | TypeKind::TyLong
            | TypeKind::TyEnum { .. } => true,
            _ => false,
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
    if let TypeKind::TyInt | TypeKind::TyShort | TypeKind::TyLong = ty.kind {
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

pub fn get_pointer_or_array_size(node: &Node) -> usize {
    match &node.ty {
        Some(ty) => match &ty.kind {
            TypeKind::TyPtr { ptr_to } | TypeKind::TyArray { ptr_to, .. } => ptr_to.size,
            _ => panic!("not a pointer or array"),
        },
        None => panic!("no type information"),
    }
}

impl Ctx<'_> {
    pub fn add_type(&mut self, node: &mut Node) {
        if node.ty.is_some() {
            return;
        }

        match &mut node.kind {
            NodeKind::NdAdd { lhs, rhs }
            | NodeKind::NdSub { lhs, rhs }
            | NodeKind::NdMul { lhs, rhs }
            | NodeKind::NdDiv { lhs, rhs }
            | NodeKind::NdMod { lhs, rhs } => {
                self.add_type(lhs);
                self.add_type(rhs);
                let ty = get_common_type(lhs.clone().ty.unwrap(), rhs.clone().ty.unwrap());
                node.ty = Some(ty);
            }
            NodeKind::NdAssign { lhs, rhs } => {
                self.add_type(lhs);
                self.add_type(rhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::NdEq { lhs, rhs }
            | NodeKind::NdNe { lhs, rhs }
            | NodeKind::NdLt { lhs, rhs }
            | NodeKind::NdLe { lhs, rhs }
            | NodeKind::NdGt { lhs, rhs }
            | NodeKind::NdGe { lhs, rhs }
            | NodeKind::NdAnd { lhs, rhs }
            | NodeKind::NdOr { lhs, rhs } => {
                self.usual_arith_conv(lhs, rhs);
                node.ty = Some(new_int_ty());
            }
            NodeKind::NdNum { val } => {
                node.ty = if *val == (*val as i32 as isize) {
                    Some(new_int_ty())
                } else {
                    Some(new_long_ty())
                }
            }
            NodeKind::NdVar { var } => {
                node.ty = Some(var.borrow().ty.clone());
            }
            NodeKind::NdAddr { lhs } => {
                self.add_type(lhs);
                if let Some(ty) = &lhs.ty {
                    node.ty = Some(new_ptr_to(ty.clone()));
                }
            }
            NodeKind::NdDeref { lhs, tok } => {
                self.add_type(lhs);
                if let Some(ty) = &lhs.ty {
                    match &ty.kind {
                        TypeKind::TyPtr { ptr_to } | TypeKind::TyArray { ptr_to, .. } => {
                            node.ty = Some((**ptr_to).clone());
                        }
                        _ => self.error_tok(&tok, "not a pointer or array"),
                    }
                } else {
                    panic!("no type information");
                }
            }
            NodeKind::NdNeg { lhs } => {
                self.add_type(lhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::NdReturn { lhs } | NodeKind::NdExprStmt { lhs } => {
                self.add_type(lhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::NdGNUStmtExpr { body } => {
                let last_node = body.last().unwrap();
                node.ty = last_node.ty.clone();
            }
            NodeKind::NdMember { member, .. } => {
                node.ty = Some(member.ty.clone()); // よくわからん
            }
            _ => {} // Block, If, For, While, Funccall
        }
    }

    pub fn usual_arith_conv(&mut self, lhs: &mut Node, rhs: &mut Node) {
        let ty = get_common_type(lhs.clone().ty.unwrap(), rhs.clone().ty.unwrap());
        *lhs = self.new_cast(lhs.clone(), ty.clone());
        *rhs = self.new_cast(rhs.clone(), ty);
    }
}
