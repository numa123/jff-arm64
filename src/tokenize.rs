use crate::types::{Node, Token, TokenKind};
use core::panic;

pub fn skip(tokens: &mut Vec<Token>, op: &str, input: &str) -> bool {
    if tokens.is_empty() {
        eprintln!("{}", input);
        eprintln!("{}^", " ".repeat(input.len()));
        eprintln!("expected {}", op);
        panic!();
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
    panic!();
}

// r_input: remaining input
// r_inputは初めの文字から順に消費され、消費されるごとに短くしていく(参照を進めることによって)
pub fn tokenize(r_input: &mut &str) -> Vec<Token> {
    let input_copy = *r_input;
    let mut tokens = Vec::new();
    let mut index = 0;
    while !r_input.is_empty() {
        let c = r_input.chars().next().unwrap();
        if c == ' ' {
            *r_input = &r_input[1..];
            index += 1;
            continue;
        }
        if c.is_digit(10) {
            let num = parse_number(r_input);
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
        if r_input.len() > 2
            && (r_input[0..2].eq("==")
                || r_input[0..2].eq("!=")
                || r_input[0..2].eq("<=")
                || r_input[0..2].eq(">="))
        {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: r_input[0..2].to_string(),
                loc: index,
            });
            *r_input = &r_input[2..];
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
            || c == ','
        {
            tokens.push(Token {
                kind: TokenKind::TkPunct,
                val: 0,
                str: r_input[0..1].to_string(),
                loc: index,
            });
            *r_input = &r_input[1..];
            index += 1;
            continue;
        }

        // 1文字以上の変数名をサポート
        if is_ident(c) {
            let mut ident = String::new();
            while !r_input.is_empty() && is_ident2(r_input.chars().next().unwrap()) {
                ident.push(r_input.chars().next().unwrap());
                *r_input = &r_input[1..];
                index += 1;
            }

            tokens.push(Token {
                kind: TokenKind::TkIdent,
                val: 0,
                str: ident,
                loc: index,
            });
            continue;
        }

        eprintln!("{}", input_copy);
        eprintln!("{}^", " ".repeat(index));
        eprintln!("invalid token");
        panic!();
    }
    return tokens;
}

// 予約後の場合はトークンの種類を変更する。
// 意味ないのではと思ったけど、変数名が予約語の場合にエラーになるために必要(多分。未確認)
pub fn convert_keywords(tokens: &mut Vec<Token>) {
    let keywords = vec!["return", "if", "else", "for", "while"]; // breakはまだ
    for t in tokens.iter_mut() {
        if keywords.contains(&t.str.as_str()) {
            t.kind = TokenKind::TkKeyword;
        }
    }
}

fn is_ident(c: char) -> bool {
    return c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_';
}

fn is_ident2(c: char) -> bool {
    return is_ident(c) || c.is_digit(10);
}
