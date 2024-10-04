use crate::types::{Node, Token, TokenKind};
use core::panic;

pub fn skip(tokens: &mut Vec<Token>, s: &str, input: &str) -> bool {
    if tokens.is_empty() {
        eprintln!("{}", input);
        eprintln!("{}^", " ".repeat(input.len()));
        eprintln!("expected {}", s);
        panic!();
    }
    if tokens[0].str != s {
        let bold_white_text = "\x1b[1m\x1b[97m"; // エラーメッセージの装飾
        let reset = "\x1b[0m";
        error_tok(
            &tokens[0],
            format!("expected {}{}{}", bold_white_text, s, reset).as_str(),
            input,
        );
    }
    tokens.remove(0);
    return true;
}

pub fn consume(tokens: &mut Vec<Token>, s: &str) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if tokens[0].str != s {
        return false;
    }
    tokens.remove(0);
    return true;
}

fn get_number(p: &mut &str) -> String {
    let num: String = p.chars().take_while(|c| c.is_digit(10)).collect();
    *p = &p[num.len()..];
    return num;
}

// 嘘だけどNodeを返すと書いている
pub fn error_tok(t: &Token, msg: &str, input: &str) -> Node {
    // エラーメッセージの装飾
    let purple_caret = "\x1b[91m^"; // 明るい紫の "^"
    let red_text = "\x1b[91m"; // 赤色の開始
    let error_header_text = "\x1b[91mjff_error:"; // 赤色の開始
    let reset = "\x1b[0m"; // 色リセット

    // t.loc文字目を赤くして表示
    let before = &input[..t.loc]; // t.loc前の部分
    let after = &input[t.loc + 1..]; // t.loc後の部分
    let target_char = input.chars().nth(t.loc).unwrap(); // t.locの文字

    eprintln!("{} {}{}", error_header_text, reset, msg);
    eprintln!("{}{}{}{}{}", before, red_text, target_char, reset, after); // t.locの文字だけ赤く
    eprintln!("{}{}{}", " ".repeat(t.loc), purple_caret, reset);
    panic!();
}

// r_input: remaining input
// r_inputは初めの文字から順に消費され、消費されるごとに短くしていく(参照を進めることによって)
pub fn tokenize(r_input: &mut &str) -> Vec<Token> {
    let input_copy = *r_input;
    let mut tokens = Vec::new();
    let mut index = 0; // debugのlocation用

    while !r_input.is_empty() {
        let c = r_input.chars().next().unwrap();
        // skip space
        if c == ' ' {
            *r_input = &r_input[1..];
            index += 1;
            continue;
        }
        // 数値
        if c.is_digit(10) {
            let num = get_number(r_input); // 数値の終わりまで取得
            tokens.push(Token {
                kind: TokenKind::TkNum,
                val: num.parse().unwrap(),
                str: num.clone(),
                loc: index,
            });
            index += num.len();
            continue;
        }
        // double character punctuator
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
        // single character punctuator
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
            || c == '&'
            || c == '['
            || c == ']'
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

        // identifier
        if is_ident(c) {
            let mut ident = String::new();
            while !r_input.is_empty() && is_ident2(r_input.chars().next().unwrap()) {
                ident.push(r_input.chars().next().unwrap());
                *r_input = &r_input[1..];
            }
            tokens.push(Token {
                kind: TokenKind::TkIdent,
                val: 0,
                str: ident.clone(),
                loc: index,
            });
            index += ident.len();
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
    let keywords = vec!["return", "if", "else", "for", "while", "int"]; // breakはまだ
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
