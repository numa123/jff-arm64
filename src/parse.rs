use std::cmp::{self};
use std::{cell::RefCell, mem::swap, rc::Rc};

use crate::tokenize::equal;
use crate::type_utils::*;
use crate::types::*;

fn align_to(n: usize, to: usize) -> usize {
    if to == 0 {
        n
    } else {
        (n + to - 1) & !(to - 1)
    }
}

//
// union, struct
//
impl Ctx<'_> {
    fn union_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::Ident { name } = &self.get_tok(0).kind {
            tag = name.clone();
            self.advance(1);
        }

        // union型の値を宣言する時;
        if !tag.is_empty() && !self.hequal("{") {
            // findtag return ty
            if let Some(ty) = self.find_tag(tag) {
                return ty;
            } else {
                self.error_tok(self.get_tok(0), "unknown struct type");
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
            kind: TypeKind::Union { members },
            size: max_size,
            align: max_align,
        };

        if !tag.is_empty() {
            self.push_tag(tag, ty.clone());
        }
        ty
    }

    fn struct_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::Ident { name } = &self.get_tok(0).kind {
            tag = name.clone();
            self.advance(1);
        }

        // 方の値を宣言する時;
        if !tag.is_empty() && !self.hequal("{") {
            // findtag return ty
            if let Some(ty) = self.find_tag(tag) {
                return ty;
            } else {
                self.error_tok(self.get_tok(0), "unknown struct type");
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
            kind: TypeKind::Struct { members },
            size: offset,
            align: max_align,
        };

        if !tag.is_empty() {
            self.push_tag(tag, ty.clone());
        }
        ty
    }

    fn struct_members(&mut self) -> Vec<Member> {
        let mut members: Vec<Member> = Vec::new();
        while !self.consume("}") {
            let base_ty = self.declspec();
            while !self.consume(";") {
                let (ty, name, _) = self.declarator(base_ty.clone());
                let member = Member {
                    name,
                    ty,
                    offset: 0, // あとでstruct_declで更新する
                };
                members.push(member);
                if self.hequal(",") {
                    self.advance(1);
                    continue;
                }
            }
        }
        members
    }

    // tyはTyStruct限定の想定
    fn get_struct_member(&mut self, ty: Type, name: String) -> Member {
        if let TypeKind::Struct { members } | TypeKind::Union { members } = ty.kind {
            for member in &members {
                if member.name == name {
                    return member.clone();
                }
            }
            self.error_tok(&self.tokens[0], "no such member");
        } else {
            self.error_tok(&self.tokens[0], "no such member"); // いらないかも
        }
    }

    fn struct_ref(&mut self, lhs: Node) -> Node {
        let mut lhs = lhs.clone();
        self.add_type(&mut lhs); // 必要か?
        if let TypeKind::Struct { .. } | TypeKind::Union { .. } = lhs.ty.as_ref().unwrap().kind {
            // nameを取り出す
            let name = if let TokenKind::Ident { name } = self.advance(1).kind {
                name
            } else {
                self.error_tok(&self.tokens[0], "expected identifier");
            };
            // nodeを作る。lhsがx.aのxの方。lhsのoffsetから、memberのoffsetを足したところのデータを取得する形になる
            let member = self.get_struct_member(copy_type(&lhs), name); // 可読性ゴミ
            Node {
                kind: NodeKind::Member {
                    lhs: Box::new(lhs.clone()),
                    member: member.clone(),
                },
                ty: Some(member.ty),
            }
        } else {
            self.error_tok(&self.tokens[0], "not a struct nor union");
        }
    }

    fn push_tag(&mut self, tag: String, ty: Type) {
        let function = self.get_func();
        let struct_tags = &mut function.scopes[function.scope_idx as usize].tags;
        let struct_tag = StructTag { tag, ty };
        struct_tags.push(struct_tag);
    }

    fn find_tag(&mut self, tag: String) -> Option<Type> {
        let function = self.get_func();
        for scope in function.scopes.iter().rev() {
            for struct_tag in &scope.tags {
                if struct_tag.tag == tag {
                    return Some(struct_tag.clone().ty);
                }
            }
        }
        None
    }
}

//
// enum
//
impl Ctx<'_> {
    fn enum_decl(&mut self) -> Type {
        let mut tag = String::new();
        if let TokenKind::Ident { name } = &self.tokens[0].kind {
            tag = name.clone();
            self.advance(1);
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
            kind: TypeKind::Enum { members: member },
            size: 4,
            align: 4,
        };
        if !tag.is_empty() {
            self.push_enum(tag, ty.clone());
        } else {
            self.push_enum("".to_string(), ty.clone());
        }
        ty
    }

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
                name,
                ty: new_int_ty(),
                val: i,
            };
            members.push(mem);
            i += 1;
            if self.consume(",") {
                continue;
            }
        }
        members
    }

    fn push_enum(&mut self, name: String, ty: Type) {
        let function = self.get_func();
        let enums = &mut function.scopes[function.scope_idx as usize].enums;
        let enm = Enum { tag: name, ty };
        enums.push(enm);
    }

    pub fn find_enum(&mut self, name: String) -> Option<Enum> {
        let func = self.get_func();
        let scopes = &func.scopes;
        for scope in scopes.iter().rev() {
            for enm in &scope.enums {
                if enm.tag == name {
                    return Some(enm.clone());
                }
            }
        }
        None
    }

    // 変数にenumのメンバを代入する際に使用
    pub fn find_enum_member(&mut self, name: &str) -> Option<Node> {
        let func = self.get_func();
        let scopes = &func.scopes;
        let mut val = None;
        for scope in scopes.iter().rev() {
            for enm in &scope.enums {
                if let TypeKind::Enum { members: list } = &enm.ty.kind {
                    for mem in list {
                        if mem.name == name {
                            val = Some(mem.val);
                        }
                    }
                }
            }
        }
        val.map(|val| self.new_num(val))
    }
}

//
// declration relation
//
impl Ctx<'_> {
    // 型のみを取得する
    // sizeof(int)や、キャストの際に使用する
    fn only_type_declarator(&mut self, ty: Type) -> Type {
        let mut ty = ty;
        while self.consume("*") {
            ty = new_ptr_to(ty);
        }
        (ty, _) = self.type_suffix(ty);
        ty
    }

    fn declspec(&mut self) -> Type {
        if self.consume("int") {
            return new_int_ty();
        } else if self.consume("short") {
            return new_short();
        } else if self.consume("long") {
            return new_long_ty();
        } else if self.consume("char") {
            return new_char_ty();
        } else if self.consume("struct") {
            return self.struct_decl();
        } else if self.consume("union") {
            return self.union_decl();
        } else if self.consume("enum") {
            return self.enum_decl();
        } else if let TokenKind::Ident { name } = &self.tokens[0].kind {
            if let Some(ty) = self.find_type(name.to_string()) {
                self.advance(1);
                return ty;
            }
        }

        if let Some(last) = self.consumed_tokens.last() {
            // ネスト小さくしたい
            if let TokenKind::Keyword { name } = &last.kind {
                if name == "typedef" {
                    return new_int_ty();
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
        (ty, name, is_func)
    }

    // 配列であればそれも含めた型を返す
    // "(" の有無で関数かの判定をする
    // return (Type, is_function)
    fn type_suffix(&mut self, ty: Type) -> (Type, bool) {
        if self.hequal("[") {
            self.advance(1);
            let size = self.get_and_skip_number();
            self.skip("]");
            let (ty, _) = self.type_suffix(ty);
            return (new_array_ty(ty, size as usize), false);
        }
        if self.hequal("(") {
            return (ty, true);
        }
        (ty, false)
    }

    fn declaration(&mut self) -> Node {
        let base_ty = self.declspec();
        let mut body = Vec::new();
        while !self.consume(";") {
            let (ty, name, _) = self.declarator(base_ty.clone());
            let mut node = self.create_lvar(name.clone().as_str(), ty, false);

            if self.hequal("=") {
                self.advance(1);
                let rhs = self.expr();
                node = self.new_assign(node, rhs);
            }
            let node = self.new_expr_stmt(node);
            body.push(node);
            if self.hequal(",") {
                self.advance(1);
                continue;
            }
        }
        self.new_block(body)
    }
}

//
// statement, expression
//
impl Ctx<'_> {
    fn stmt(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::Keyword { name } if name == "return" => {
                self.advance(1);
                let expr = self.expr();
                let node = self.new_return(expr);
                self.skip(";");
                return node;
            }
            TokenKind::Keyword { name } if name == "if" => {
                self.advance(1);
                let node: Node;
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let then = self.stmt();
                let mut els = None;
                if self.hequal("else") {
                    self.advance(1);
                    els = Some(self.stmt());
                }
                node = self.new_if(cond, then, els);
                return node;
            }
            TokenKind::Keyword { name } if name == "for" => {
                self.advance(1);
                self.enter_scope();
                self.skip("(");
                let init = if self.is_typename(&self.tokens[0].clone()) {
                    self.declaration()
                } else {
                    self.expr_stmt()
                };
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
                self.leave_scope();
                return node;
            }
            TokenKind::Keyword { name } if name == "while" => {
                self.advance(1);
                self.skip("(");
                let cond = self.expr();
                self.skip(")");
                let body = self.stmt();
                let node = self.new_while(cond, body);
                return node;
            }
            TokenKind::Punct { str } if str == "{" => {
                self.skip("{");
                let node = self.compound_stmt();
                return node;
            }
            _ => {}
        }

        self.expr_stmt()
    }

    fn push_type(&mut self, deftype: TypedefType) {
        let func = self.get_func();
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
                self.advance(1);
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
        self.new_block(body)
    }

    fn expr_stmt(&mut self) -> Node {
        if self.hequal(";") {
            self.advance(1);
            return self.null_stmt();
        }
        let expr = self.expr();
        let node = self.new_expr_stmt(expr);
        self.skip(";");
        node
    }

    fn expr(&mut self) -> Node {
        self.assign()
    }

    fn assign(&mut self) -> Node {
        let mut node = self.bit();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "=" => {
                    self.advance(1);
                    let assign = self.assign();
                    node = self.new_assign(node, assign);
                }
                TokenKind::Punct { str } if str == "+=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let add = self.new_add(node.clone(), assign);
                    node = self.new_assign(node, add);
                }
                TokenKind::Punct { str } if str == "-=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let sub = self.new_sub(node.clone(), assign);
                    node = self.new_assign(node, sub);
                }
                TokenKind::Punct { str } if str == "*=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let mul = self.new_mul(node.clone(), assign);
                    node = self.new_assign(node, mul);
                }
                TokenKind::Punct { str } if str == "/=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let div = self.new_div(node.clone(), assign);
                    node = self.new_assign(node, div);
                }
                TokenKind::Punct { str } if str == "%=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let mod_ = self.new_mod(node.clone(), assign);
                    node = self.new_assign(node, mod_);
                }
                TokenKind::Punct { str } if str == "&=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let bit_and = self.new_bit_and(node.clone(), assign);
                    node = self.new_assign(node, bit_and);
                }
                TokenKind::Punct { str } if str == "^=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let bit_xor = self.new_bit_xor(node.clone(), assign);
                    node = self.new_assign(node, bit_xor);
                }
                TokenKind::Punct { str } if str == "|=" => {
                    self.advance(1);
                    let assign = self.assign();
                    let bit_or = self.new_bit_or(node.clone(), assign);
                    node = self.new_assign(node, bit_or);
                }
                _ => break,
            }
        }
        node
    }

    fn bit(&mut self) -> Node {
        let mut node = self.equality();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "&" => {
                    self.advance(1);
                    let equality = self.equality();
                    node = self.new_bit_and(node, equality);
                }
                TokenKind::Punct { str } if str == "^" => {
                    self.advance(1);
                    let equality = self.equality();
                    node = self.new_bit_xor(node, equality);
                }
                TokenKind::Punct { str } if str == "|" => {
                    self.advance(1);
                    let equality = self.equality();
                    node = self.new_bit_or(node, equality);
                }
                _ => break,
            }
        }
        node
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "||" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_or(node, add);
                }
                TokenKind::Punct { str } if str == "&&" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_and(node, add);
                }
                TokenKind::Punct { str } if str == "==" => {
                    self.advance(1);
                    let relational = self.relational();
                    node = self.new_eq(node, relational);
                }
                TokenKind::Punct { str } if str == "!=" => {
                    self.advance(1);
                    let relational = self.relational();
                    node = self.new_ne(node, relational);
                }
                _ => break,
            }
        }
        node
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "<" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_lt(node, add);
                }
                TokenKind::Punct { str } if str == "<=" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_le(node, add);
                }
                TokenKind::Punct { str } if str == ">" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_gt(node, add);
                }
                TokenKind::Punct { str } if str == ">=" => {
                    self.advance(1);
                    let add = self.add();
                    node = self.new_ge(node, add);
                }
                _ => break,
            }
        }
        node
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "+" => {
                    self.advance(1);
                    let rhs = self.mul();
                    node = self.new_add(node, rhs);
                }
                TokenKind::Punct { str } if str == "-" => {
                    self.advance(1);
                    let rhs = self.mul();
                    node = self.new_sub(node, rhs);
                }
                _ => break,
            }
        }
        node
    }

    fn new_add(&mut self, lhs: Node, rhs: Node) -> Node {
        let mut lhs = lhs;
        let mut rhs = rhs;
        self.add_type(&mut lhs);
        self.add_type(&mut rhs);
        // num + num
        if is_integer_node(&lhs) && is_integer_node(&rhs) {
            let mut node = Node {
                kind: NodeKind::Add {
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
            let num = self.new_long(size as isize);
            let r = self.new_mul(rhs, num);
            let node = Node {
                kind: NodeKind::Add {
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
                kind: NodeKind::Sub {
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
                kind: NodeKind::Sub {
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(r),
                },
                ty: Some(lhs.ty.clone().unwrap()),
            };
            return node;
        }
        // ptr - ptr
        if is_pointer_node(&lhs) && is_pointer_node(&rhs) {
            let div_size = if let TypeKind::Ptr { ptr_to } = copy_type(&rhs).kind {
                ptr_to.size
            } else {
                8 // とりあえず8
            };
            let mut n = Node {
                kind: NodeKind::Sub {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ty: None,
            };
            self.add_type(&mut n);
            let num = self.new_long(div_size as isize);
            let mut node = self.new_div(n, num);
            let ty = new_int_ty();
            node.ty = Some(ty);
            return node;
        }
        self.error_tok(&self.tokens[0], "invalid operands");
    }

    fn mul(&mut self) -> Node {
        let mut node = self.cast();
        while !self.tokens.is_empty() {
            match &self.tokens[0].kind {
                TokenKind::Punct { str } if str == "*" => {
                    self.advance(1);
                    let cast = self.cast();
                    node = self.new_mul(node, cast);
                }
                TokenKind::Punct { str } if str == "/" => {
                    self.advance(1);
                    let cast = self.cast();
                    return self.new_div(node, cast);
                }
                TokenKind::Punct { str } if str == "%" => {
                    self.advance(1);
                    let cast = self.cast();
                    return self.new_mod(node, cast);
                }
                _ => break,
            }
        }
        node
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
        self.unary()
    }

    fn unary(&mut self) -> Node {
        if self.hequal("+") {
            self.advance(1);
            return self.cast();
        }
        if self.hequal("-") {
            self.advance(1);
            let cast = self.cast();
            return self.new_neg(cast);
        }
        if self.hequal("&") {
            self.advance(1);
            let cast = self.cast();
            return self.new_addr(cast);
        }
        if self.hequal("*") {
            self.advance(1);
            let cast = self.cast();
            return self.new_deref(cast, self.tokens[0].clone());
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Node {
        let mut node = self.primary();
        loop {
            if self.hequal("[") {
                self.advance(1);
                let idx = self.expr();
                self.skip("]");
                let add = self.new_add(node, idx);
                node = self.new_deref(add, self.tokens[0].clone());
                continue;
            }
            if self.hequal(".") {
                self.advance(1);
                node = self.struct_ref(node);
                continue;
            }
            if self.hequal("->") {
                let tok = self.advance(1);
                node = self.new_deref(node, tok);
                node = self.struct_ref(node);
                continue;
            }
            if self.hequal("++") {
                self.advance(1);
            }
            break;
        }
        self.add_type(&mut node);
        node
    }

    fn primary(&mut self) -> Node {
        match &self.tokens[0].kind {
            TokenKind::Num { .. } => {
                let num = self.get_and_skip_number();
                self.new_num(num)
            }
            // gnu statement expression
            TokenKind::Punct { str } if str == "(" && equal(&self.tokens[1].clone(), "{") => {
                self.advance(2);
                let compound_stmt = self.compound_stmt();
                let body = match compound_stmt.kind {
                    NodeKind::Block { body } => body,
                    _ => unreachable!("gnu statement expression body is not block"),
                };
                let mut node = Node {
                    kind: NodeKind::GNUStmtExpr { body },
                    ty: None,
                };
                self.add_type(&mut node);
                self.skip(")");
                node
            }
            TokenKind::Punct { str } if str == "(" => {
                self.advance(1);
                let node = self.expr();
                self.skip(")");
                node
            }
            TokenKind::Keyword { name } if name == "sizeof" => {
                self.advance(1);
                // sizeof(type)
                if self.hequal("(") && self.is_typename(&self.tokens[1].clone()) {
                    self.advance(1);
                    let ty = self.typename();
                    self.skip(")");
                    return self.new_num(ty.size as isize);
                }
                // sizeof(ident)
                let mut node = self.unary();
                self.add_type(&mut node);
                self.new_num(copy_type(&node).size as isize)
            }
            TokenKind::Str { str } => {
                let name = format!("lC{}", self.gvars.len());
                let var = self.create_gvar(
                    name.as_str(),
                    new_array_ty(new_char_ty(), str.len()),
                    Some(InitGval::Str(str.clone())),
                );
                self.advance(1);
                var
            }
            TokenKind::Ident { name } => {
                let name = name.clone();
                self.advance(1);
                let node: Node;
                // funccall
                if self.hequal("(") {
                    node = self.funccall(&name);
                    return node;
                }

                // variable
                // まずローカル変数、グローバル変数から取得しようとし、そこになければenumのメンバを探す
                if let Some(var) = self.find_var(&name) {
                    node = Node {
                        kind: NodeKind::Var { var: var.clone() }, // Clone the Rc to increase the reference count
                        ty: Some(copy_var_type(&var)),
                    };
                } else if let Some(n) = self.find_enum_member(&name) {
                    node = n;
                } else {
                    self.error_tok(self.get_tok(-1), "undefined variable");
                    // -1しないといけない
                }
                node
            }

            _ => self.error_tok(
                &self.tokens[0],
                r#"expected a value-returning expression. Maybe a "}" was left out? "#,
            ),
        }
    }

    // ここ、関数のtyも返すようにしたいが、自分で定義したものだけで、includeしたものをどうするかわからん
    fn funccall(&mut self, name: &str) -> Node {
        self.advance(1);
        let mut args = Vec::new();
        while !self.consume(")") {
            if self.hequal(",") {
                self.advance(1);
            }
            args.push(self.assign());
        }
        if args.len() > 8 {
            self.error_tok(
                self.consumed_tokens.last().unwrap(),
                "too many arguments. 8 arguments are allowed at most",
            );
        }
        Node {
            kind: NodeKind::FuncCall {
                name: name.to_string(),
                args,
            },
            ty: Some(new_long_ty()), // 自分で定義するようになったら、また変数リストから、型を取り出して入れる。includeの場合はどうする？足し算とかできないよな。まあ後で考えるか。一旦chibiccにならってlong
        }
    }
}

//
// main process
//
impl Ctx<'_> {
    pub fn new_func(&mut self, name: &str, ty: Type) {
        // 関数の場合
        // これから処理する関数名をセット。create_lvar, find_varで使用
        self.cur_func = name.to_string();
        let func = self.create_func(name, ty);
        self.functions.insert(name.to_string(), func);

        self.enter_scope();

        // 引数の処理
        self.skip("(");
        while !self.consume(")") {
            let base_ty = self.declspec();
            let (ty, name, _) = self.declarator(base_ty);
            self.create_lvar(name.as_str(), ty, true);
            self.consume(","); // ,があればスキップ、なければ何もしないで、whileの条件分で終了
        }

        if self.consume(";") {
            let func = self.get_func();
            func.is_def = false;
            return;
        }

        self.skip("{");

        // 関数の中身の処理
        self.functions.get_mut(name).unwrap().body = Some(self.compound_stmt());
    }

    fn get_func(&mut self) -> &mut Function {
        return self.functions.get_mut(&self.cur_func).unwrap();
    }

    pub fn enter_scope(&mut self) {
        let func = self.get_func();

        func.scopes.push(Scope {
            variables: Vec::new(),
            tags: Vec::new(),
            types: Vec::new(),
            enums: Vec::new(),
        });
        func.scope_idx += 1;
    }

    pub fn leave_scope(&mut self) {
        let func = self.get_func();
        let poped_scope = func.scopes.pop();
        func.exited_scope.push(poped_scope.unwrap());
        func.scope_idx -= 1;
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
        self.tokens = self.tokenize();
        self.convert_keywords();

        // グローバル変数の定義文をwhileで回す
        while !self.tokens.is_empty() {
            let base_ty = self.declspec();
            let (ty, name, is_func) = self.declarator(base_ty.clone());

            // 関数ではない場合
            if !is_func {
                self.new_gvar(name.as_str(), base_ty, ty);
                continue;
            }
            // 関数の場合
            self.new_func(name.as_str(), ty);
            self.leave_scope();
        }
    }
}

// utility
impl Ctx<'_> {
    // variable
    fn create_gvar(&mut self, name: &str, ty: Type, init_gval: Option<InitGval>) -> Node {
        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: self.gvars.len(), // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            ty: ty.clone(),
            is_param: false,
            is_local: false,
            init_gval,
        }));

        self.gvars.push(var.clone());
        self.new_var(var)
    }

    // 関数定義用の
    fn create_lvar(&mut self, name: &str, ty: Type, is_def_arg: bool) -> Node {
        // eprintln!("呼ばれた: {:#?}", &self.tokens[0]);
        let function = self.get_func();

        let var = Rc::new(RefCell::new(Var {
            name: name.to_string(),
            offset: 0, // Offset will be calculated later // あとで方を実装した際、そのsizeなりによって変更すべき。ここでやるか、codegenでやるかはあとで
            // ここcodegenで改めてoffsetを割り当てているから意味ない説。グローバル変数はまた別で使えるかも。名前とか
            ty: ty.clone(),
            is_param: is_def_arg,
            is_local: true,
            init_gval: None,
        }));

        function.scopes[function.scope_idx as usize]
            .variables
            .push(var.clone()); // コードがひどいが、まあ想定ではここが-になることはないはず

        let mut node = self.new_var(var);
        // 関数定義の引数の場合、関数のargsにも追加
        if is_def_arg {
            let func = self.functions.get_mut(&self.cur_func).unwrap();
            if func.args.len() >= 8 {
                self.error_tok(
                    self.consumed_tokens.last().unwrap(),
                    "too many arguments. 8 arguments are allowed at most",
                );
            }
            func.args.push(node.clone());
        }
        self.add_type(&mut node);
        node
    }

    pub fn find_var(&mut self, name: &str) -> Option<Rc<RefCell<Var>>> {
        for scope in self.get_func().scopes.iter().rev() {
            for var in scope.variables.iter() {
                if var.borrow().name == name {
                    return Some(var.clone());
                }
            }
        }
        for var in &self.gvars {
            if var.borrow_mut().name == name {
                // なぜborrow_mut()なのか
                return Some(var.clone());
            }
        }
        None
    }

    // type
    fn find_type(&mut self, name: String) -> Option<Type> {
        let func = self.get_func();
        for scope in func.scopes.iter().rev() {
            for deftype in &scope.types {
                if deftype.name == name {
                    return Some(deftype.ty.clone());
                }
            }
        }
        None
    }

    // cast, sizeofの際に使用
    fn typename(&mut self) -> Type {
        let base_ty = self.declspec();
        self.only_type_declarator(base_ty)
    }

    fn is_typename(&mut self, tok: &Token) -> bool {
        match &tok.kind {
            TokenKind::Keyword { name } => {
                matches!(
                    name.as_str(),
                    "int" | "short" | "long" | "char" | "struct" | "union" | "enum"
                )
            }
            TokenKind::Ident { name } => self.find_type(name.to_string()).is_some(),
            _ => false,
        }
    }

    fn get_ident(&mut self) -> String {
        let n: String;
        if let TokenKind::Ident { name } = &self.tokens[0].kind {
            n = name.clone();
        } else {
            self.error_tok(&self.tokens[0], "expected identifier");
        }
        self.advance(1);
        n
    }

    fn get_num(&mut self) -> isize {
        let n: isize;
        if let TokenKind::Num { val } = &self.tokens[0].kind {
            n = *val;
        } else {
            self.error_tok(&self.tokens[0], "expected number");
        }
        self.advance(1);
        n
    }
}
