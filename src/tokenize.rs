use crate::types::*;

impl<'a> Ctx<'a> {
    // 入力: 数字から始まる文字列　出力: 数字列。副作用: 文字列を数値の次の文字列まで進める
    pub fn parse_and_skip_number(&mut self) -> isize {
        let num: String = self
            .input
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        self.input = &self.input[num.len()..];
        num.parse().unwrap()
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
                while !self.input.is_empty() && self.input.starts_with('\n') {
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

            if c.is_ascii_digit() {
                let num = self.parse_and_skip_number();
                tokens.push(Token {
                    kind: TokenKind::Num { val: num },
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
                || self.input.starts_with("++")
                || self.input.starts_with("--")
            {
                tokens.push(Token {
                    kind: TokenKind::Punct {
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
                    kind: TokenKind::Punct { str: c.to_string() },
                    start: self.current_input_position(),
                    len: 1,
                });
                self.advance_input(1);
                continue;
            }

            if c == '"' {
                let mut str = String::new();
                self.advance_input(1);
                // while self.input.chars().next().unwrap() != '"' {
                while !self.input.starts_with('\"') {
                    // 取り急ぎの実装。後でなんとかしないと。string.cのテストに対応したいがために実装したもの。
                    if self.input.starts_with("\\\"") {
                        str.push_str("\\\"");
                        self.advance_input(2); // \"が入るとstrのlenがずれて、tok.startもずれるかも
                        continue;
                    }
                    str.push(self.input.chars().next().unwrap());
                    self.advance_input(1);
                }
                str.push('\0');
                self.advance_input(1);
                tokens.push(Token {
                    kind: TokenKind::Str { str: str.clone() },
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
                    kind: TokenKind::Ident { name: name.clone() },
                    start: self.current_input_position() - name.len(),
                    len: name.len(),
                });
                continue;
            }
            self.error_input_at(format!("invalid input: {}", &self.input[0..1]).as_str());
        }
        tokens
    }

    pub fn convert_keywords(&mut self) {
        let keywords = vec![
            "return", "if", "else", "for", "while", "int", "sizeof", "char", "struct", "union",
            "long", "short", "typedef", "enum",
        ];
        for token in &mut self.tokens {
            if let TokenKind::Ident { name } = &token.kind {
                if keywords.contains(&name.as_str()) {
                    token.kind = TokenKind::Keyword { name: name.clone() };
                }
            }
        }
    }
}

fn is_ident(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
}
fn is_ident2(c: char) -> bool {
    is_ident(c) || c.is_ascii_digit()
}

pub fn equal(tok: &Token, s: &str) -> bool {
    if let TokenKind::Punct { str } = &tok.kind {
        return str == s;
    }
    if let TokenKind::Keyword { name } = &tok.kind {
        return name == s;
    }
    false
}

// for token
impl Ctx<'_> {
    pub fn get_and_skip_number(&mut self) -> isize {
        match self.tokens[0].kind {
            TokenKind::Num { val } => {
                self.consumed_tokens.push(self.tokens.remove(0));
                val
            }
            _ => self.error_tok(&self.tokens[0], "expected a number"),
        }
    }

    // nが0以上なら、未処理のトークンからの取得。負ならすでに処理されたトークンからの取得とする。
    // もっとも最近処理されたトークンへのアクセスは、self.get_tok(-1);
    pub fn get_tok(&self, n: isize) -> &Token {
        if n >= 0 {
            &self.tokens[0]
        } else {
            // lenが0で、nが負の場合エラーになるが、本コードではそのような使い方はしない
            &self.consumed_tokens[((self.consumed_tokens.len() as isize) + n) as usize]
        }
    }

    // 複数個飛ばす場合、最後のtokenのみが返る
    pub fn advance(&mut self, n: usize) -> Token {
        assert!(n >= 1);
        let mut tok = self.tokens.remove(0);
        self.consumed_tokens.push(tok.clone());
        for _ in 0..(n - 1) {
            tok = self.tokens.remove(0);
            self.consumed_tokens.push(tok.clone());
        }
        tok
    }

    pub fn skip(&mut self, op: &str) -> Token {
        if let TokenKind::Punct { str } = &self.tokens[0].kind {
            if str == op {
                let tok = self.tokens.remove(0);
                self.consumed_tokens.push(tok.clone());
                return tok;
            }
        }
        if let TokenKind::Keyword { name } = &self.tokens[0].kind {
            if name == op {
                let tok = self.tokens.remove(0);
                self.consumed_tokens.push(tok.clone());
                return tok;
            }
        }
        self.error_tok(self.get_tok(0), format!("expected '{}'", op).as_str())
    }

    pub fn hequal(&mut self, s: &str) -> bool {
        if let TokenKind::Punct { str } = &self.tokens[0].kind {
            return str == s;
        }
        if let TokenKind::Keyword { name } = &self.tokens[0].kind {
            return name == s;
        }
        false
    }

    pub fn consume(&mut self, s: &str) -> bool {
        if self.hequal(s) {
            self.consumed_tokens.push(self.tokens.remove(0));
            true
        } else {
            false
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

        eprintln!("{}:{}: error", self.cur_file, line_idx);
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
