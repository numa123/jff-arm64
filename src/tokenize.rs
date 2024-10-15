use crate::types::*;

impl<'a> Ctx<'a> {
    // 入力: 数字から始まる文字列　出力: 数字列。副作用: 文字列を数値の次の文字列まで進める
    pub fn parse_and_skip_number(&mut self) -> isize {
        let num: String = self.input.chars().take_while(|c| c.is_digit(10)).collect();
        self.input = &self.input[num.len()..];
        return num.parse().unwrap();
    }
}
impl Ctx<'_> {
    pub fn advance_input(&mut self, n: usize) {
        self.input = &self.input[n..];
    }
    pub fn current_input_position(&self) -> usize {
        self.input_copy.len() - self.input.len()
    }
}

// for token
impl Ctx<'_> {
    pub fn get_and_skip_number(&mut self) -> isize {
        match self.tokens[0].kind {
            TokenKind::TkNum { val } => {
                self.tokens.remove(0);
                return val;
            }
            _ => self.error_tok(&self.tokens[0], "expected a number"),
        }
    }

    pub fn advance_one_tok(&mut self) -> Token {
        self.tokens.remove(0)
    }

    #[allow(dead_code)]
    pub fn advance_toks(&mut self, n: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        for _ in 0..n {
            tokens.push(self.tokens.remove(0));
        }
        tokens
    }

    pub fn skip(&mut self, op: &str) -> Token {
        if let TokenKind::TkPunct { str } = &self.tokens[0].kind {
            if str == op {
                return self.tokens.remove(0);
            }
        }
        if let TokenKind::TkKeyword { name } = &self.tokens[0].kind {
            if name == op {
                return self.tokens.remove(0);
            }
        }
        self.error_tok(&self.tokens[0], format!("expected '{}'", op).as_str())
    }

    pub fn equal(&mut self, op: &str) -> bool {
        if let TokenKind::TkPunct { str } = &self.tokens[0].kind {
            return str == op;
        }
        if let TokenKind::TkKeyword { name } = &self.tokens[0].kind {
            return name == op;
        }
        false
    }

    pub fn consume(&mut self, s: &str) -> bool {
        if self.equal(s) {
            self.tokens.remove(0);
            return true;
        } else {
            return false;
        }
    }

    #[allow(dead_code)]
    pub fn show_tokens(&self) {
        eprintln!("{:#?}", self.tokens);
    }

    pub fn error_tok(&self, tok: &Token, msg: &str) -> ! {
        let mut idx = 0;
        let mut line_idx = 1;
        let mut line_string_before = String::new();
        let mut line_string = String::new();
        for line in self.input_copy.lines() {
            if idx + line.len() >= tok.start {
                line_string = line.to_string();
                break;
            }
            idx += line.len() + 1;
            line_idx += 1;
            line_string_before = line.to_string();
        }

        eprintln!("{}:{}: error", self.processing_filename, line_idx);
        eprintln!();
        eprintln!("|");
        eprintln!("|{}", line_string_before);
        eprintln!("|{}", line_string);
        eprintln!(
            "|{}{} {}",
            " ".repeat(tok.start - idx),
            "^".repeat(tok.len),
            msg
        ); // 後々該当箇所のinput_copyを色付けして表す
        eprintln!("|");
        eprintln!();
        panic!();
    }

    pub fn error_input_at(&self, msg: &str) {
        eprintln!("{}", self.input_copy);
        eprintln!("{}^", " ".repeat(self.current_input_position()));
        eprintln!("jff_error: {}", msg);
        panic!();
    }
}

impl Ctx<'_> {
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.input.is_empty() {
            let c = self.input.chars().next().unwrap();

            if c == ' ' {
                self.advance_input(1);
                continue;
            }

            // 改行文字のスキップ
            if c == '\n' {
                self.advance_input(1);
                continue;
            }

            // 行コメントのスキップ
            if self.input.starts_with("//") {
                self.advance_input(2);
                while !self.input.is_empty() && self.input.chars().next().unwrap() != '\n' {
                    self.advance_input(1);
                }
                continue;
            }

            // ブロックコメントのスキップ
            if self.input.starts_with("/*") {
                self.advance_input(2);
                while !self.input.starts_with("*/") {
                    self.advance_input(1);
                }
                self.advance_input(2);
                continue;
            }

            if c.is_digit(10) {
                let num = self.parse_and_skip_number();
                tokens.push(Token {
                    kind: TokenKind::TkNum { val: num },
                    start: self.current_input_position() - num.to_string().len(),
                    len: num.to_string().len(),
                });
                continue;
            }
            if self.input.starts_with("==")
                || self.input.starts_with("!=")
                || self.input.starts_with(">=")
                || self.input.starts_with("<=")
                || self.input.starts_with("&&")
                || self.input.starts_with("||")
                || self.input.starts_with("+=")
                || self.input.starts_with("-=")
                || self.input.starts_with("*=")
                || self.input.starts_with("/=")
                || self.input.starts_with("%=")
                || self.input.starts_with("&=")
                || self.input.starts_with("^=")
                || self.input.starts_with("|=")
                || self.input.starts_with("->")
            {
                tokens.push(Token {
                    kind: TokenKind::TkPunct {
                        str: self.input[0..2].to_string(),
                    },
                    start: self.current_input_position(),
                    len: 2,
                });
                self.advance_input(2);
                continue;
            }
            if c == '+'
                || c == '-'
                || c == '*'
                || c == '/'
                || c == '('
                || c == ')'
                || c == '>'
                || c == '<'
                || c == ';'
                || c == '='
                || c == '{'
                || c == '}'
                || c == '&'
                || c == ','
                || c == '['
                || c == ']'
                || c == '%'
                || c == '^'
                || c == '|'
                || c == '.'
            {
                tokens.push(Token {
                    kind: TokenKind::TkPunct { str: c.to_string() },
                    start: self.current_input_position(),
                    len: 1,
                });
                self.advance_input(1);
                continue;
            }

            if c == '"' {
                let mut str = String::new();
                self.advance_input(1);
                while self.input.chars().next().unwrap() != '"' {
                    // 取り急ぎの実装。後でなんとかしないと。string.cのテストに対応したいがために実装したもの。
                    if self.input.starts_with("\\\"") {
                        str.push_str("\\\"");
                        self.advance_input(2); // \"が入るとstrのlenがずれて、tok.startもずれるかも
                        continue;
                    }
                    str.push(self.input.chars().next().unwrap());
                    self.advance_input(1);
                }
                str.push_str("\0");
                self.advance_input(1);
                tokens.push(Token {
                    kind: TokenKind::TkStr { str: str.clone() },
                    start: self.current_input_position() - str.len(),
                    len: str.len(),
                });
                continue;
            }

            // identifier
            if is_ident(c) {
                let name: String = self.input.chars().take_while(|c| is_ident2(*c)).collect();
                self.advance_input(name.len());

                tokens.push(Token {
                    kind: TokenKind::TkIdent { name: name.clone() },
                    start: self.current_input_position() - name.len(),
                    len: name.len(),
                });
                continue;
            }
            self.error_input_at(
                format!("invalid input: {}", self.input[0..1].to_string()).as_str(),
            );
        }
        return tokens;
    }

    pub fn convert_keywords(&mut self) {
        let keywords = vec![
            "return", "if", "else", "for", "while", "int", "sizeof", "char", "struct", "union",
            "long",
        ];
        for token in &mut self.tokens {
            if let TokenKind::TkIdent { name } = &token.kind {
                if keywords.contains(&name.as_str()) {
                    token.kind = TokenKind::TkKeyword { name: name.clone() };
                }
            }
        }
    }
}

fn is_ident(c: char) -> bool {
    return c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_';
}
fn is_ident2(c: char) -> bool {
    return is_ident(c) || c.is_digit(10);
}
