use crate::types::{Node, Token, TokenKind};

pub fn skip(tokens: &mut Vec<Token>, op: &str, input: &str) -> bool {
    if tokens.is_empty() {
        eprintln!("{}", input);
        eprintln!("{}^", " ".repeat(input.len()));
        eprintln!("expected {}", op);
        std::process::exit(1);
    }
    if tokens[0].str != op {
        error_tok(&tokens[0], format!("expected {}", op).as_str(), input); // as_str()は&strに変換するためのもので、to_string()はStringに変換するためのもの
    }
    tokens.remove(0);
    return true;
}

fn parse_number(p: &mut &str) -> String {
    let num: String = p.chars().take_while(|c| c.is_digit(10)).collect();
    *p = &p[num.len()..]; // これは関数の外に出した方が明示的に書きやすいかも？
    return num;
}

// 嘘だけどNodeを返すと書いている
pub fn error_tok(t: &Token, msg: &str, input: &str) -> Node {
    eprintln!("{}", input);
    eprintln!("{}^", " ".repeat(t.loc));
    eprintln!("{}", msg);
    std::process::exit(1);
}

pub fn tokenize(p: &mut &str) -> Vec<Token> {
    let p_copy = *p;
    let mut tokens = Vec::new();
    let mut index = 0;
    while !p.is_empty() {
        let c = p.chars().next().unwrap();
        if c == ' ' {
            *p = &p[1..];
            index += 1;
            continue;
        }
        if c.is_digit(10) {
            let num = parse_number(p);
            tokens.push(Token {
                kind: TokenKind::TkNum,
                val: num.parse().unwrap(),
                str: num.clone(),
                loc: index,
            });
            index += num.len();
            continue;
        }
        // ==, !=, <=, >= p.len() > 2 がないとindex out of boundsになる
        if p.len() > 2
            && (p[0..2].eq("==") || p[0..2].eq("!=") || p[0..2].eq("<=") || p[0..2].eq(">="))
        {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: p[0..2].to_string(),
                loc: index,
            });
            *p = &p[2..];
            index += 2;
            continue;
        }

        if c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '('
            || c == ')'
            || c == '<'
            || c == '>'
            || c == ';'
            || c == '='
            || c == '{'
            || c == '}'
        {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: p[0..1].to_string(),
                loc: index,
            });
            *p = &p[1..];
            index += 1;
            continue;
        }

        // 一文字の識別子のみをサポート
        if c >= 'a' && c <= 'z' {
            let mut ident = String::new();
            while !p.is_empty() && is_ident(p.chars().next().unwrap()) {
                ident.push(p.chars().next().unwrap());
                *p = &p[1..];
                index += 1;
            }

            tokens.push(Token {
                kind: TokenKind::TkIdent,
                val: 0,
                str: ident, // Stringじゃなくて&strの方が良いのかもしれない？
                loc: index,
            });
            continue;
        }

        eprintln!("{}", p_copy);
        eprintln!("{}^", " ".repeat(index));
        eprintln!("invalid token");
        std::process::exit(1);
    }
    // println!("{:?}", tokens); デバッグ用
    return tokens;
}

// キーワードを変換するためのもの。これ今は要らなくね？
pub fn convert_keywords(tokens: &mut Vec<Token>) {
    let keywords = vec!["return", "if", "else", "for"]; // breakは？
    for t in tokens.iter_mut() {
        if keywords.contains(&t.str.as_str()) {
            t.kind = TokenKind::TkKeyword;
        }
    }
}

fn is_ident(c: char) -> bool {
    return c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_';
}
