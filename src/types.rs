use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Ctx<'a> {
    pub input: &'a str,
    pub input_copy: &'a str,
    pub tokens: Vec<Token>,
    pub consumed_tokens: Vec<Token>,
    pub gvars: Vec<Rc<RefCell<Var>>>, // find_varのために型をrefcellにしてみる。不適切の恐れあり
    pub cur_func: String,
    pub cur_file: String,
    pub functions: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Function {
    #[allow(dead_code)]
    pub name: String, // 一応つけている方が自然だと思ってつけている。
    pub body: Option<Node>, // {compound_stmt}
    pub args: Vec<Node>,    // Vec<Rc<RefCell<Var>>>にするかも。可変長引数の場合。
    #[allow(dead_code)]
    pub ty: Type, // 一応つけている方が自然だと思ってつけている。関数の返り値の型が必要なケースがあるときに使うのではと思っている。 // includeしたやつとかがどういう扱いになっているのかわからないといけないと思う
    pub scopes: Vec<Scope>,
    pub scope_idx: isize,
    pub exited_scope: Vec<Scope>,
    pub is_def: bool,
}

#[derive(Debug)]
pub struct Scope {
    pub variables: Vec<Rc<RefCell<Var>>>,
    pub tags: Vec<StructTag>,
    pub types: Vec<TypedefType>,
    pub enums: Vec<Enum>, // ただしEnum
}
#[derive(Debug)]
pub struct TypedefType {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct StructTag {
    pub tag: String,
    pub ty: Type,
}

// strucgtagとほぼ同じ。でも分けた方が考えやすい
#[derive(Debug, Clone)]
pub struct Enum {
    pub tag: String,
    pub ty: Type, // size, alignはてきとーというか、もはやいらないなあ。まあいっか
}

//
// token
//
#[derive(Debug, Clone)]
pub enum TokenKind {
    Punct { str: String },
    Num { val: isize },
    Ident { name: String },
    Keyword { name: String },
    Str { str: String },
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

//
// variable
//
#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub offset: usize,
    pub ty: Type,
    #[allow(dead_code)]
    pub is_param: bool, // 8個を超える引数を扱う際、スタックを利用して引数を渡すことになると思うので、その実装の際に必要になる想定
    pub is_local: bool,
    pub init_gval: Option<InitGval>,
}

#[derive(Debug, Clone)]
pub enum InitGval {
    Str(String),
    #[allow(dead_code)]
    Num(isize), // 初期化の際に必要だけど./test.shとかで少し邪魔だからdead_codeにしている
}

//
// node
//
#[derive(Debug, Clone)]
pub enum NodeKind {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Sub {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Mul {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Div {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Mod {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Neg {
        lhs: Box<Node>,
    },
    Eq {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Ne {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Lt {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Le {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Gt {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Ge {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    And {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Or {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    ExprStmt {
        lhs: Box<Node>,
    },
    Num {
        val: isize,
    },
    NdAssign {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Var {
        var: Rc<RefCell<Var>>,
    },
    Return {
        lhs: Box<Node>,
    },
    Block {
        body: Vec<Node>,
    },
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    },
    For {
        init: Box<Node>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        body: Box<Node>,
    },
    While {
        cond: Box<Node>,
        body: Box<Node>,
    },
    Addr {
        lhs: Box<Node>,
    },
    Deref {
        lhs: Box<Node>,
        tok: Token,
    },
    FuncCall {
        name: String,
        args: Vec<Node>,
    },
    GNUStmtExpr {
        body: Vec<Node>, // compound_stmt
    },
    BitAnd {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    BitXor {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    BitOr {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Member {
        lhs: Box<Node>,
        member: Member,
    },
    Cas {
        lhs: Box<Node>,
    },
}

// for struct and union
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

//
// type
//
#[derive(Debug, Clone)]
pub enum TypeKind {
    Int,
    Short,
    Long,
    Ptr {
        ptr_to: Box<Type>,
    },
    #[allow(dead_code)]
    Array {
        ptr_to: Box<Type>,
        len: usize, // <- これをまだ未使用だからdead_codeにしている, lenがある方が自然だおと持っている
    },
    Char,
    Struct {
        members: Vec<Member>,
    },
    Union {
        members: Vec<Member>,
    },
    Enum {
        members: Vec<EnumMember>,
    },
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    #[allow(dead_code)]
    pub ty: Type, //Int
    pub val: isize, // value
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub size: usize,
    pub align: usize,
}
