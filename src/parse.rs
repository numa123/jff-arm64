use std::cmp::{self};
use std::{cell::RefCell, mem::swap, rc::Rc};

use crate::type_utils::*;
use crate::types::*;

impl Ctx<'_> {
    fn create_func(&mut self, name: &str, ty: Type) -> Function {
        let func = Function {
            name: name.to_string(),
            args: Vec::new(),
            body: None,
            ty: ty,
            scopes: Vec::new(),
            scope_idx: -1, // 最初のスコープは-1にすることで、enter_scopeで良い感じに辻褄合わせ。でも、普通にわかりづらいから後で直す
            exited_scope: Vec::new(),
            is_def: true,
        };
        return func;
    }

    fn new_if(&mut self, cond: Node, then: Node, els: Option<Node>) -> Node {
        let mut node = Node {
            kind: NodeKind::NdIf {
                cond: Box::new(cond),
                then: Box::new(then),
                els: els.map(Box::new),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_for(&mut self, init: Node, cond: Option<Node>, inc: Option<Node>, body: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdFor {
                init: Box::new(init),
                cond: cond.map(Box::new),
                inc: inc.map(Box::new),
                body: Box::new(body),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_while(&mut self, cond: Node, body: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdWhile {
                cond: Box::new(cond),
                body: Box::new(body),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_block(&mut self, body: Vec<Node>) -> Node {
        let mut node = Node {
            kind: NodeKind::NdBlock { body },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn null_stmt(&self) -> Node {
        return Node {
            kind: NodeKind::NdBlock { body: Vec::new() },
            ty: None,
        };
    }

    fn new_expr_stmt(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdExprStmt { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_assign(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdAssign {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_return(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdReturn { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_eq(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdEq {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_ne(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdNe {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_lt(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdLt {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_le(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdLe {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_gt(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdGt {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_ge(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdGe {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_and(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdAnd {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_or(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdOr {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_neg(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdNeg { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_addr(&mut self, lhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdAddr { lhs: Box::new(lhs) },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_deref(&mut self, lhs: Node, tok: Token) -> Node {
        let mut node = Node {
            kind: NodeKind::NdDeref {
                lhs: Box::new(lhs),
                tok: tok,
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_mul(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdMul {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_div(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdDiv {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: Some(new_int()),
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_mod(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdMod {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: Some(new_int()),
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_num(&mut self, val: isize) -> Node {
        let mut node = Node {
            kind: NodeKind::NdNum { val },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_var(&mut self, var: Rc<RefCell<Var>>) -> Node {
        let mut node = Node {
            kind: NodeKind::NdVar { var },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_bit_and(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdBitAnd {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_bit_xor(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdBitXor {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_bit_or(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut node = Node {
            kind: NodeKind::NdBitOr {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty: None,
        };
        self.add_type(&mut node);
        return node;
    }

    fn new_cast(&mut self, lhs: Node, ty: Type) -> Node {
        let mut node = Node {
            kind: NodeKind::NdCast { lhs: Box::new(lhs) },
            ty: Some(ty),
        };
        self.add_type(&mut node);
        return node;
    }
}

fn align_to(n: usize, to: usize) -> usize {
    if to == 0 {
        return n; // なぜreturnを書く必要がある？ nではだめなのか
    }
    (n + to - 1) & !(to - 1)
}

// struct relation
// 今は、struct {int a;} x;  みたいな宣言の仕方
impl Ctx<'_> {
    fn push_tag(&mut self, tag: String, ty: Type) {
        let function = self.get_function();
        let struct_tags = &mut function.scopes[function.scope_idx as usize].tags;
        let struct_tag = StructTag { tag, ty };
        struct_tags.push(struct_tag);
    }

    fn find_tag(&mut self, tag: String) -> Option<Type> {
        let function = self.get_function();
        for scope in function.scopes.iter().rev() {
            for struct_tag in &scope.tags {
                if struct_tag.tag == tag {
                    return Some(struct_tag.clone().ty);
                }
            }
        }
        return None;
    }

    fn union_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            tag = name.clone();
            self.advance_one_tok();
        }

        // 方の値を宣言する時;
        if !tag.is_empty() && !self.hequal("{") {
            // findtag return ty
            if let Some(ty) = self.find_tag(tag) {
                return ty;
            } else {
                self.error_tok(&self.tokens[0], "unknown struct type");
            }
        }

        self.skip("{");
        let mut members = self.struct_members();
        let mut max_size = 0;
        let mut max_align = 0;
        for member in &mut members {
            max_size = cmp::max(max_size, member.ty.size);
            max_align = cmp::max(max_align, member.ty.align);
        }
        if max_align > 8 {
            max_align = 8;
        }
        max_size = align_to(max_size, max_align);
        let ty = Type {
            kind: TypeKind::TyUnion { members: members },
            size: max_size,
            align: max_align,
        };

        if !tag.is_empty() {
            self.push_tag(tag, ty.clone());
        }
        return ty;
    }

    fn struct_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            tag = name.clone();
            self.advance_one_tok();
        }

        // 方の値を宣言する時;
        if !tag.is_empty() && !self.hequal("{") {
            // findtag return ty
            if let Some(ty) = self.find_tag(tag) {
                return ty;
            } else {
                self.error_tok(&self.tokens[0], "unknown struct type");
            }
        }

        self.skip("{");
        let mut members = self.struct_members();
        let mut offset = 0;
        // alignmentを加える
        let mut max_align = 0;
        for member in &mut members {
            offset = align_to(offset, member.ty.align);
            max_align = cmp::max(max_align, member.ty.align);
            member.offset = offset;
            offset += member.ty.size;
        }
        if max_align > 8 {
            max_align = 8;
        }
        offset = align_to(offset, max_align);
        let ty = Type {
            kind: TypeKind::TyStruct { members: members },
            size: offset,
            align: max_align,
        };

        if !tag.is_empty() {
            self.push_tag(tag, ty.clone());
        }
        return ty;
    }

    fn struct_members(&mut self) -> Vec<Member> {
        let mut members: Vec<Member> = Vec::new();
        while !self.consume("}") {
            let base_ty = self.declspec();
            while !self.consume(";") {
                let (ty, name, _) = self.declarator(base_ty.clone());
                let member = Member {
                    name: name,
                    ty: ty,
                    offset: 0, // あとでstruct_declで更新する
                };
                members.push(member);
                if self.hequal(",") {
                    self.advance_one_tok();
                    continue;
                }
            }
        }
        return members;
    }

    // tyはTyStruct限定の想定
    fn get_struct_member(&mut self, ty: Type, name: String) -> Member {
        if let TypeKind::TyStruct { members } | TypeKind::TyUnion { members } = ty.kind {
            for member in &members {
                if member.name == name {
                    return member.clone();
                }
            }
            self.error_tok(&self.tokens[0], "no such member");
        } else {
            self.error_tok(&self.tokens[0], "no such member");
        }
    }

    fn struct_ref(&mut self, lhs: Node) -> Node {
        let mut lhs = lhs.clone();
        self.add_type(&mut lhs); // 必要か?
        if let TypeKind::TyStruct { .. } | TypeKind::TyUnion { .. } = lhs.ty.as_ref().unwrap().kind
        {
            // nameを取り出す
            let name = if let TokenKind::TkIdent { name } = self.advance_one_tok().kind {
                name
            } else {
                self.error_tok(&self.tokens[0], "expected identifier");
            };
            // nodeを作る。lhsがx.aのxの方。lhsのoffsetから、memberのoffsetを足したところのデータを取得する形になる
            let member = self.get_struct_member(lhs.clone().ty.unwrap().clone(), name); // 可読性ゴミ
            let node = Node {
                kind: NodeKind::NdMember {
                    lhs: Box::new(lhs.clone()),
                    member: member.clone(),
                },
                ty: Some(member.ty),
            };
            return node;
        } else {
            self.error_tok(&self.tokens[0], "not a struct nor union");
        }
    }
}

impl Ctx<'_> {
    fn enum_members(&mut self) -> Vec<EnumMember> {
        let mut members: Vec<EnumMember> = Vec::new();
        let mut i = 0;
        while !self.consume("}") {
            let name = self.get_ident();
            // 代入
            if self.consume("=") {
                i = self.get_num();
            }
            let mem = EnumMember {
                name: name,
                ty: new_int(),
                val: i,
            };
            members.push(mem);
            i += 1;
            if self.consume(",") {
                continue;
            }
        }
        return members;
    }

    fn enum_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            tag = name.clone();
            self.advance_one_tok();
        }

        if !tag.is_empty() && !self.hequal("{") {
            if let Some(enm) = self.find_enum(tag) {
                return enm.ty;
            } else {
                self.error_tok(&self.tokens[0], "unknown enum type");
            }
        }

        self.skip("{");
        let member = self.enum_members();
        let ty = Type {
            kind: TypeKind::TyEnum { list: member },
            size: 4,
            align: 4,
        };
        if !tag.is_empty() {
            self.push_enum(tag, ty.clone());
        } else {
            self.push_enum("".to_string(), ty.clone());
        }
        return ty;
    }

    fn push_enum(&mut self, name: String, ty: Type) {
        let function = self.get_function();
        let enums = &mut function.scopes[function.scope_idx as usize].enums;
        let enm = Enum { name: name, ty: ty };
        enums.push(enm);
    }

    pub fn find_enum(&mut self, name: String) -> Option<Enum> {
        let func = self.get_function();
        let scopes = &func.scopes;
        for scope in scopes.iter().rev() {
            for enm in &scope.enums {
                if enm.name == name {
                    return Some(enm.clone());
                }
            }
        }
        return None;
    }

    pub fn find_enum_primary(&mut self, name: &str) -> Option<Node> {
        let func = self.get_function();
        let scopes = &func.scopes;
        let mut val = None;
        for scope in scopes.iter().rev() {
            for enm in &scope.enums {
                if let TypeKind::TyEnum { list } = &enm.ty.kind {
                    for mem in list {
                        if mem.name == name {
                            val = Some(mem.val);
                        }
                    }
                }
            }
        }
        if let Some(val) = val {
            return Some(self.new_num(val));
        } else {
            return None;
        }
    }
}

impl Ctx<'_> {
    fn only_type_declarator(&mut self, ty: Type) -> Type {
        let mut ty = ty;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        (ty, _) = self.type_suffix(ty);
        return ty;
    }
    fn typename(&mut self) -> Type {
        let base_ty = self.declspec();
        let ty = self.only_type_declarator(base_ty);
        return ty;
    }
    fn declspec(&mut self) -> Type {
        if self.consume("int") {
            return new_int();
        } else if self.consume("short") {
            return new_short();
        } else if self.consume("long") {
            return new_long();
        } else if self.consume("char") {
            return new_char();
        } else if self.consume("struct") {
            return self.struct_decl();
        } else if self.consume("union") {
            return self.union_decl();
        } else if self.consume("enum") {
            return self.enum_decl();
        } else if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            if let Some(ty) = self.find_type(name.to_string()) {
                self.advance_one_tok();
                return ty;
            }
        }

        if let Some(last) = self.exited_tokens.last() {
            // ネスト小さくしたい
            if let TokenKind::TkKeyword { name } = &last.kind {
                if name == "typedef" {
                    return new_int();
                }
            }
        }
        self.error_tok(&self.tokens[0], "type expected");
    }

    // return (Type, name, is_function)
    // グローバル変数か、関数かの判定に使う。parseのはじめ以外では使用しない
    fn declarator(&mut self, ty: Type) -> (Type, String, bool) {
        let mut ty = ty;
        let is_func: bool;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        let name = self.get_ident();
        (ty, is_func) = self.type_suffix(ty);
        return (ty, name, is_func);
    }

    // 今は配列のみ
    // return (Type, is_function)
    fn type_suffix(&mut self, ty: Type) -> (Type, bool) {
        if self.hequal("[") {
            self.advance_one_tok();
            let size = self.get_and_skip_number();
            self.skip("]");
            let (ty, _) = self.type_suffix(ty);
            return (new_array_ty(ty, size as usize), false);
        }
        if self.hequal("(") {
            return (ty, true);
        }
        return (ty, false);
    }

    fn declaration(&mut self) -> Node {
        let base_ty = self.declspec();
        let mut body = Vec::new();
        while !self.hequal(";") {
            let (ty, name, _) = self.declarator(base_ty.clone());
            let mut node = self.create_lvar(name.clone().as_str(), ty, false);

            if self.hequal("=") {
                self.advance_one_tok();
                let rhs = self.expr();
                node = self.new_assign(node, rhs);
            }
            let node = self.new_expr_stmt(node);
            body.push(node);
            if self.hequal(",") {
                self.advance_one_tok();
                continue;
            }
        }
        let node = self.new_block(body);
        self.skip(";");
        return node;
    }

    fn find_type(&mut self, name: String) -> Option<Type> {
        let func = self.get_function();
        for scope in func.scopes.iter().rev() {
            for deftype in &scope.types {
                if deftype.name == name {
                    return Some(deftype.ty.clone());
                }
            }
        }
        return None;
    }

    fn is_typename(&mut self, tok: &Token) -> bool {
        match &tok.kind {
            TokenKind::TkKeyword { name } => match name.as_str() {
                "int" | "short" | "long" | "char" | "struct" | "union" | "enum" => return true,
                _ => return false,
            },
            TokenKind::TkIdent { name } => {
                if let Some(_) = self.find_type(name.to_string()) {
                    return true;
                } else {
                    false
                }
            }
            _ => return false,
        }
    }

    fn get_ident(&mut self) -> String {
        let n: String;
        if let TokenKind::TkIdent { name } = &self.tokens[0].kind {
            n = name.clone();
        } else {
            self.error_tok(&self.tokens[0], "expected identifier");
        }
        self.advance_one_tok();
        return n;
    }

    fn get_num(&mut self) -> isize {
        let n: isize;
        if let TokenKind::TkNum { val } = &self.tokens[0].kind {
            n = *val;
        } else {
            self.error_tok(&self.tokens[0], "expected number");
        }
        self.advance_one_tok();
        return n;
    }
}

impl Ctx<'_> {
    fn stmt(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkKeyword { name } if name == "return" => {
                self.advance_one_tok();
                let expr = self.expr();
                let node = self.new_return(expr);
                self.skip(";");
                return node;
            }
            TokenKind::TkKeyword { name } if name == "if" => {
                self.advance_one_tok();
                let node: Node;
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let then = self.stmt();
                let mut els = None;
                if self.hequal("else") {
                    self.advance_one_tok();
                    els = Some(self.stmt());
                }
                node = self.new_if(cond, then, els);
                return node;
            }
            TokenKind::TkKeyword { name } if name == "for" => {
                self.advance_one_tok();
                self.skip("(");
                let init = self.expr_stmt();
                let mut cond = None;
                let mut inc = None;
                if !self.hequal(";") {
                    cond = Some(self.expr());
                }
                self.skip(";");
                if !self.hequal(")") {
                    inc = Some(self.expr());
                }
                self.skip(")");
                let body = self.stmt();
                let node = self.new_for(init, cond, inc, body);
                return node;
            }
            TokenKind::TkKeyword { name } if name == "while" => {
                self.advance_one_tok();
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let body = self.stmt();
                let node = self.new_while(cond, body);
                return node;
            }
            TokenKind::TkPunct { str } if str == "{" => {
                self.skip("{");
                let node = self.compound_stmt();
                return node;
            }
            _ => {}
        }

        return self.expr_stmt();
    }

    fn push_type(&mut self, deftype: TypedefType) {
        let func = self.get_function();
        let types = &mut func.scopes[func.scope_idx as usize].types;
        types.push(deftype);
    }

    fn compound_stmt(&mut self) -> Node {
        let mut body = Vec::new();
        self.enter_scope();
        while !self.consume("}") {
            if self.is_typename(&self.tokens[0].clone()) {
                let mut node = self.declaration();
                self.add_type(&mut node);
                body.push(node);
            } else if self.hequal("typedef") {
                // typedefはstmtだが、Nodeを返さないという点で特殊なのでここに書いた
                self.advance_one_tok();
                let base_ty = self.declspec();
                let (ty, name, _) = self.declarator(base_ty);
                let deftype = TypedefType {
                    name: name.clone(),
                    ty,
                };
                self.push_type(deftype);
                self.skip(";");
            } else {
                let mut stmt = self.stmt();
                self.add_type(&mut stmt);
                body.push(stmt);
            }
        }
        self.leave_scope();
        let node = self.new_block(body);
        return node;
    }

    fn expr_stmt(&mut self) -> Node {
        if self.hequal(";") {
            self.advance_one_tok();
            return self.null_stmt();
        }
        let expr = self.expr();
        let node = self.new_expr_stmt(expr);
        self.skip(";");
        return node;
    }

    fn expr(&mut self) -> Node {
        return self.assign();
    }

    fn assign(&mut self) -> Node {
        let mut node = self.bit();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    node = self.new_assign(node, assign);
                }
                TokenKind::TkPunct { str } if str == "+=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let add = self.new_add(node.clone(), assign);
                    node = self.new_assign(node, add);
                }
                TokenKind::TkPunct { str } if str == "-=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let sub = self.new_sub(node.clone(), assign);
                    node = self.new_assign(node, sub);
                }
                TokenKind::TkPunct { str } if str == "*=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let mul = self.new_mul(node.clone(), assign);
                    node = self.new_assign(node, mul);
                }
                TokenKind::TkPunct { str } if str == "/=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let div = self.new_div(node.clone(), assign);
                    node = self.new_assign(node, div);
                }
                TokenKind::TkPunct { str } if str == "%=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let mod_ = self.new_mod(node.clone(), assign);
                    node = self.new_assign(node, mod_);
                }
                TokenKind::TkPunct { str } if str == "&=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let bit_and = self.new_bit_and(node.clone(), assign);
                    node = self.new_assign(node, bit_and);
                }
                TokenKind::TkPunct { str } if str == "^=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let bit_xor = self.new_bit_xor(node.clone(), assign);
                    node = self.new_assign(node, bit_xor);
                }
                TokenKind::TkPunct { str } if str == "|=" => {
                    self.advance_one_tok();
                    let assign = self.assign();
                    let bit_or = self.new_bit_or(node.clone(), assign);
                    node = self.new_assign(node, bit_or);
                }
                _ => break,
            }
        }
        return node;
    }

    fn bit(&mut self) -> Node {
        let mut node = self.equality();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "&" => {
                    self.advance_one_tok();
                    let equality = self.equality();
                    node = self.new_bit_and(node, equality);
                }
                TokenKind::TkPunct { str } if str == "^" => {
                    self.advance_one_tok();
                    let equality = self.equality();
                    node = self.new_bit_xor(node, equality);
                }
                TokenKind::TkPunct { str } if str == "|" => {
                    self.advance_one_tok();
                    let equality = self.equality();
                    node = self.new_bit_or(node, equality);
                }
                _ => break,
            }
        }
        return node;
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "||" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_or(node, add);
                }
                TokenKind::TkPunct { str } if str == "&&" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_and(node, add);
                }
                TokenKind::TkPunct { str } if str == "==" => {
                    self.advance_one_tok();
                    let relational = self.relational();
                    node = self.new_eq(node, relational);
                }
                TokenKind::TkPunct { str } if str == "!=" => {
                    self.advance_one_tok();
                    let relational = self.relational();
                    node = self.new_ne(node, relational);
                }
                _ => break,
            }
        }
        return node;
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "<" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_lt(node, add);
                }
                TokenKind::TkPunct { str } if str == "<=" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_le(node, add);
                }
                TokenKind::TkPunct { str } if str == ">" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_gt(node, add);
                }
                TokenKind::TkPunct { str } if str == ">=" => {
                    self.advance_one_tok();
                    let add = self.add();
                    node = self.new_ge(node, add);
                }
                _ => break,
            }
        }
        return node;
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "+" => {
                    self.advance_one_tok();
                    let rhs = self.mul();
                    node = self.new_add(node, rhs);
                }
                TokenKind::TkPunct { str } if str == "-" => {
                    self.advance_one_tok();
                    let rhs = self.mul();
                    node = self.new_sub(node, rhs);
                }
                _ => break,
            }
        }
        return node;
    }

    fn new_add(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut lhs = lhs;
        let mut rhs = rhs;
        self.add_type(&mut lhs);
        self.add_type(&mut rhs);
        // num + num
        if is_integer_node(&lhs) && is_integer_node(&rhs) {
            let mut node = Node {
                kind: NodeKind::NdAdd {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ty: None,
            };
            self.add_type(&mut node);
            return node;
        }
        if is_pointer_node(&lhs) && is_pointer_node(&rhs) {
            self.error_tok(&self.tokens[0], "cannot add pointers to pointers");
        }
        // canonicalize num + ptr -> ptr + num
        if is_integer_node(&lhs) && is_pointer_node(&rhs) {
            swap(&mut lhs, &mut rhs);
        }
        // ptr + num
        if is_pointer_node(&lhs) && is_integer_node(&rhs) {
            // node.tyのkindのptr_toのsizeを取得してvalに足す
            let size = get_pointer_or_array_size(&lhs);
            let num = self.new_num(size as isize);
            let r = self.new_mul(rhs, num);
            let node = Node {
                kind: NodeKind::NdAdd {
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(r),
                },
                ty: Some(lhs.ty.clone().unwrap()),
            };
            return node;
        }
        eprintln!("lhs type: {:#?}", lhs.ty); // ここはaが入っていなければならないのに
        eprintln!("rhs type: {:#?}", rhs.ty);
        self.error_tok(&self.tokens[0], "invalid operands");
    }

    fn new_sub(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut lhs = lhs;
        let mut rhs = rhs;
        self.add_type(&mut lhs);
        self.add_type(&mut rhs);
        // num - num
        if is_integer_node(&lhs) && is_integer_node(&rhs) {
            let mut node = Node {
                kind: NodeKind::NdSub {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ty: None,
            };
            self.add_type(&mut node);
            return node;
        }
        // ptr - num
        if is_pointer_node(&lhs) && is_integer_node(&rhs) {
            let size = get_pointer_or_array_size(&lhs);
            let num = self.new_num(size as isize);
            let r = self.new_mul(rhs, num);
            let node = Node {
                kind: NodeKind::NdSub {
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(r),
                },
                ty: Some(lhs.ty.clone().unwrap()),
            };
            return node;
        }
        // ptr - ptr
        if is_pointer_node(&lhs) && is_pointer_node(&rhs) {
            let div_size = if let TypeKind::TyPtr { ptr_to } = &lhs.clone().ty.unwrap().kind {
                ptr_to.size
            } else {
                8 // とりあえず8
            };
            let mut n = Node {
                kind: NodeKind::NdSub {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ty: None,
            };
            self.add_type(&mut n);
            let num = self.new_num(div_size as isize);
            let mut node = self.new_div(n, num);
            let ty = new_int();
            node.ty = Some(ty);
            return node;
        }
        self.error_tok(&self.tokens[0], "invalid operands");
    }

    fn mul(&mut self) -> Node {
        let mut node = self.cast();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::TkPunct { str } if str == "*" => {
                    self.advance_one_tok();
                    let cast = self.cast();
                    node = self.new_mul(node, cast);
                }
                TokenKind::TkPunct { str } if str == "/" => {
                    self.advance_one_tok();
                    let cast = self.cast();
                    return self.new_div(node, cast);
                }
                TokenKind::TkPunct { str } if str == "%" => {
                    self.advance_one_tok();
                    let cast = self.cast();
                    return self.new_mod(node, cast);
                }
                _ => break,
            }
        }
        return node;
    }

    fn cast(&mut self) -> Node {
        if self.hequal("(") && self.is_typename(&self.tokens[1].clone()) {
            self.skip("(");
            let ty = self.typename();
            self.skip(")");
            let cast = self.cast();
            let node = self.new_cast(cast, ty);
            return node;
        }
        return self.unary();
    }

    fn unary(&mut self) -> Node {
        if self.hequal("+") {
            self.advance_one_tok();
            return self.cast();
        }
        if self.hequal("-") {
            self.advance_one_tok();
            let cast = self.cast();
            return self.new_neg(cast);
        }
        if self.hequal("&") {
            self.advance_one_tok();
            let cast = self.cast();
            return self.new_addr(cast);
        }
        if self.hequal("*") {
            self.advance_one_tok();
            let cast = self.cast();
            return self.new_deref(cast, self.tokens[0].clone());
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Node {
        let mut node = self.primary();
        loop {
            if self.hequal("[") {
                self.advance_one_tok();
                let idx = self.expr();
                self.skip("]");
                let add = self.new_add(node, idx);
                node = self.new_deref(add, self.tokens[0].clone());
                continue;
            }
            if self.hequal(".") {
                self.advance_one_tok();
                node = self.struct_ref(node);
                continue;
            }
            if self.hequal("->") {
                let tok = self.advance_one_tok();
                node = self.new_deref(node, tok);
                node = self.struct_ref(node);
                continue;
            }
            break;
        }
        self.add_type(&mut node);
        return node;
    }

    fn primary(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::TkNum { .. } => {
                let num = self.get_and_skip_number();
                return self.new_num(num);
            }
            TokenKind::TkPunct { str } if str == "(" => {
                self.advance_one_tok();
                // gnu statement expression
                if self.hequal("{") {
                    self.advance_one_tok();
                    let compound_stmt = self.compound_stmt();
                    let body = if let NodeKind::NdBlock { body } = compound_stmt.kind {
                        body
                    } else {
                        panic!("gnu statement expression body is not block");
                    };
                    let mut node = Node {
                        kind: NodeKind::NdGNUStmtExpr { body: body },
                        ty: None,
                    };
                    self.add_type(&mut node);
                    self.skip(")");
                    return node;
                }
                let node = self.expr();
                self.skip(")");
                return node;
            }
            TokenKind::TkKeyword { name } if name == "sizeof" => {
                self.advance_one_tok();
                // sizeof(type)
                if self.hequal("(") && self.is_typename(&self.tokens[1].clone()) {
                    self.skip("(");
                    let ty = self.typename();
                    self.skip(")");
                    return self.new_num(ty.size as isize); //
                }

                // sizeof(ident)
                let mut node = self.unary();
                self.add_type(&mut node);
                return self.new_num(node.ty.as_ref().unwrap().size as isize);
            }
            TokenKind::TkStr { str } => {
                let name = format!("lC{}", self.global_variables.len());
                let var = self.create_gvar(
                    name.as_str(),
                    new_array_ty(new_char(), str.len()),
                    Some(InitGval::Str(str.clone())),
                );
                self.advance_one_tok();
                return var;
            }
            TokenKind::TkIdent { name } => {
                let name = name.clone();
                self.advance_one_tok();
                let node: Node;
                // funccall
                if self.hequal("(") {
                    node = self.funccall(&name);
                    return node;
                }

                // variable
                if let Some(var) = self.find_var(&name) {
                    node = Node {
                        kind: NodeKind::NdVar { var: var.clone() }, // Clone the Rc to increase the reference count
                        ty: Some(var.borrow().clone().ty),
                    };
                } else if let Some(n) = self.find_enum_primary(&name) {
                    node = n;
                } else {
                    self.error_tok(&self.tokens[0], "undefined variable");
                    // -1しないといけない
                }
                return node;
            }

            _ => self.error_tok(
                &self.tokens[0],
                r#"expected a value-returning expression. Maybe a "}" was left out? "#,
            ),
        }
    }

    // ここ、関数のtyも返すようにしたいが、自分で定義したものだけで、includeしたものをどうするかわからん
    fn funccall(&mut self, name: &str) -> Node {
        self.advance_one_tok();
        let mut args = Vec::new();
        while !self.hequal(")") {
            if self.hequal(",") {
                self.advance_one_tok();
            }
            args.push(self.assign());
        }
        // これは巻き戻さないといけないエラー
        if args.len() > 8 {
            self.error_tok(
                &self.tokens[0],
                "too many arguments. 8 arguments are allowed at most",
            );
        }
        let node = Node {
            kind: NodeKind::NdFuncCall {
                name: name.to_string(),
                args: args,
            },
            ty: Some(new_long()), // 自分で定義するようになったら、また変数リストから、型を取り出して入れる。includeの場合はどうする？足し算とかできないよな。まあ後で考えるか。一旦chibiccにならってlong
        };
        self.skip(")");
        return node;
    }
}

impl Ctx<'_> {
    fn get_function(&mut self) -> &mut Function {
        return self.functions.get_mut(&self.processing_funcname).unwrap();
    }

    pub fn enter_scope(&mut self) {
        let function = self.get_function();

        //
        function.scopes.push(Scope {
            variables: Vec::new(),
            tags: Vec::new(),
            types: Vec::new(),
            enums: Vec::new(),
        });
        function.scope_idx += 1;
    }
    pub fn leave_scope(&mut self) {
        let function = self.get_function();
        let poped_scope = function.scopes.pop();
        function.exited_scope.push(poped_scope.unwrap());
        function.scope_idx -= 1;
    }

    fn create_gvar(&mut self, name: &str, ty: Type, init_gval: Option<InitGval>) -> Node {
        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: self.global_variables.len(), // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            ty: ty.clone(),
            is_def_arg: false,
            is_local: false,
            init_gval: init_gval,
        }));

        self.global_variables.push(var.clone());
        return self.new_var(var);
    }

    // 関数定義用の
    fn create_lvar(&mut self, name: &str, ty: Type, is_def_arg: bool) -> Node {
        // eprintln!("呼ばれた: {:#?}", &self.tokens[0]);
        let function = self.get_function();

        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: 0, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            // ここcodegenで改めてoffsetを割り当てているから意味ない説。グローバル変数はまた別で使えるかも。名前とか
            ty: ty.clone(),
            is_def_arg: is_def_arg,
            is_local: true,
            init_gval: None,
        }));

        function.scopes[function.scope_idx as usize]
            .variables
            .push(var.clone()); // コードがひどいが、まあ想定ではここが-になることはないはず

        let mut node = self.new_var(var);
        // 関数定義の引数の場合、関数のargsにも追加
        if is_def_arg {
            let func = self.functions.get_mut(&self.processing_funcname).unwrap();
            if func.args.len() >= 8 {
                self.error_tok(
                    &self.tokens[0],
                    "too many arguments. 8 arguments are allowed at most",
                );
            }
            func.args.push(node.clone());
        }
        self.add_type(&mut node);
        return node;
    }

    pub fn find_var(&mut self, name: &str) -> Option<Rc<RefCell<Var>>> {
        let function = self.get_function();
        let scopes = &function.scopes; // なぜ&が必要なのか。
        for scope in scopes.iter().rev() {
            for var in scope.variables.iter() {
                if var.borrow().name == name {
                    return Some(var.clone());
                }
            }
        }
        for var in &self.global_variables {
            if var.borrow_mut().name == name {
                // なぜborrow_mut()なのか
                return Some(var.clone());
            }
        }
        return None;
    }
}

impl Ctx<'_> {
    pub fn new_function(&mut self, name: &str, ty: Type) {
        // 関数の場合
        // これから処理する関数名をセット。create_lvar, find_varで使用
        self.processing_funcname = name.to_string();
        let func = self.create_func(name, ty);
        self.functions.insert(name.to_string(), func);

        self.enter_scope();

        // 引数の処理
        self.skip("(");
        while !self.hequal(")") {
            let base_ty = self.declspec();
            let (ty, name, _) = self.declarator(base_ty);
            self.create_lvar(name.as_str(), ty, true);
            self.consume(","); // ,があればスキップ、なければ何もしないで、whileの条件分で終了
        }
        self.skip(")");

        if self.consume(";") {
            let func = self.get_function();
            func.is_def = false;
            return;
        }

        self.skip("{");

        // 関数の中身の処理
        // これからはローカル変数の処理という意味合い
        self.is_processing_local = true;
        self.functions.get_mut(name).unwrap().body = Some(self.compound_stmt());
    }

    pub fn new_gvar(&mut self, name: &str, base_ty: Type, ty: Type) {
        self.create_gvar(name, ty, None);
        while self.consume(",") {
            let (ty, name, _) = self.declarator(base_ty.clone()); // なぜclone
            self.create_gvar(name.as_str(), ty, None);
        }
        self.skip(";");
    }

    pub fn parse(&mut self) {
        // inputをトークンに変換
        self.tokens = self.tokenize();
        self.convert_keywords();

        // グローバル変数の定義文をwhileで回す
        while !self.tokens.is_empty() {
            // これからグローバル変数の定義を行うという意味合い
            self.is_processing_local = false;

            let base_ty = self.declspec();
            let (ty, name, is_func) = self.declarator(base_ty.clone());

            // 関数ではない場合
            if !is_func {
                self.new_gvar(name.as_str(), base_ty, ty);
                continue;
            }
            // 関数の場合
            self.new_function(name.as_str(), ty);
            self.leave_scope();
        }
    }
}
