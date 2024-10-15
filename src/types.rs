use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Ctx<'a> {
    pub input: &'a str,
    pub input_copy: &'a str,
    pub tokens: Vec<Token>,
    pub global_variables: Vec<Rc<RefCell<Var>>>, // find_varのために型をrefcellにしてみる。不適切の恐れあり
    pub processing_funcname: String,
    pub processing_filename: String,
    pub is_processing_local: bool, // グローバル変数の定義をしているのか、ローカル変数の定義をしているのかどうか
    pub functions: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Scope {
    pub variables: Vec<Rc<RefCell<Var>>>,
    pub tags: Vec<StructTag>,
}

#[derive(Debug)]
pub struct Function {
    #[allow(dead_code)]
    pub name: String, // 一応つけている方が自然だと思ってつけている。
    pub body: Option<Node>, // {compound_stmt}
    pub args: Vec<Node>,    // Vec<Rc<RefCell<Var>>>にするかも。可変長引数の場合。
    #[allow(dead_code)]
    pub ty: Type, // 一応つけている方が自然だと思ってつけている。関数の返り値の型が必要なケースがあるときに使うのではと思っている。includeしたやつとかがどういう扱いになっているのかわからないといけないと思う
    pub scopes: Vec<Scope>,
    pub scope_idx: isize,
    pub exited_scope: Vec<Scope>,
}

#[derive(Debug, Clone)]
pub struct StructTag {
    pub tag: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    TkPunct { str: String },
    TkNum { val: isize },
    TkIdent { name: String },
    TkKeyword { name: String },
    TkStr { str: String },
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub enum InitGval {
    Str(String),
    #[allow(dead_code)]
    Num(isize), // 初期化の際に必要だけど./test.shとかで少し邪魔だからdead_codeにしている
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub offset: usize,
    pub ty: Type,
    #[allow(dead_code)]
    pub is_def_arg: bool, // 8個を超える引数を扱う際、スタックを利用して引数を渡すことになると思うので、その実装の際に必要になる想定
    pub is_local: bool,
    pub init_gval: Option<InitGval>,
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
    NdMod {
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
    NdAnd {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdOr {
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
        tok: Token,
    },
    NdFuncCall {
        name: String,
        args: Vec<Node>,
    },
    NdGNUStmtExpr {
        body: Vec<Node>, // compound_stmt
    },
    NdBitAnd {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdBitXor {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdBitOr {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    NdMember {
        lhs: Box<Node>,
        member: Member,
    },
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub ty: Type,
    pub offset: usize, // 構造体からの相対オフセット
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    TyInt,
    TyShort,
    TyLong,
    TyPtr {
        ptr_to: Box<Type>,
    },
    #[allow(dead_code)]
    TyArray {
        ptr_to: Box<Type>,
        len: usize, // <- これをまだ未使用だからdead_codeにしている, lenがある方が自然だおと持っている
    },
    TyChar,
    TyStruct {
        members: Vec<Member>,
    },
    TyUnion {
        members: Vec<Member>,
    },
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub size: usize,
    pub align: usize,
}
