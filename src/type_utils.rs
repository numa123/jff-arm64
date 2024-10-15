use crate::types::*;

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

pub fn new_char() -> Type {
    Type {
        kind: TypeKind::TyChar,
        size: 1,
    }
}

pub fn new_struct(members: Vec<Member>, size: usize) -> Type {
    Type {
        kind: TypeKind::TyStruct { members: members },
        size: size,
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
        match ty.kind {
            TypeKind::TyInt => true,
            TypeKind::TyChar => true, // charと数値は足せる
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
            | NodeKind::NdMod { lhs, rhs }
            | NodeKind::NdAssign { lhs, rhs } => {
                self.add_type(lhs);
                self.add_type(rhs);
                node.ty = lhs.ty.clone();
            }
            NodeKind::NdEq { .. }
            | NodeKind::NdNe { .. }
            | NodeKind::NdLt { .. }
            | NodeKind::NdLe { .. }
            | NodeKind::NdGt { .. }
            | NodeKind::NdGe { .. }
            | NodeKind::NdAnd { .. }
            | NodeKind::NdOr { .. }
            | NodeKind::NdNum { .. } => {
                node.ty = Some(new_int());
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
            NodeKind::NdReturn { lhs } | NodeKind::NdExprStmt { lhs } | NodeKind::NdNeg { lhs } => {
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
            _ => {} // Block, If, For, While
        }
    }
}
