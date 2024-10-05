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

    pub fn advance_tok(&mut self, n: usize) {
        for _ in 0..n {
            self.tokens.remove(0);
        }
    }

    pub fn skip(&mut self, op: &str) -> Token {
        if let TokenKind::TkPunct { str } = &self.tokens[0].kind {
            if str == op {
                return self.tokens.remove(0);
            }
        }
        self.error_tok(&self.tokens[0], format!("expected '{}'", op).as_str())
    }

    pub fn equal(&mut self, op: &str) -> bool {
        if let TokenKind::TkPunct { str } = &self.tokens[0].kind {
            return str == op;
        }
        false
    }

    pub fn error_tok(&self, tok: &Token, msg: &str) -> ! {
        eprintln!("{}", self.input_copy);
        eprintln!("{}{}", " ".repeat(tok.start), "^".repeat(tok.len)); // 後々該当箇所のinput_copyを色付けして表す
        eprintln!("jff_error: {}", msg);
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
            {
                tokens.push(Token {
                    kind: TokenKind::TkPunct { str: c.to_string() },
                    start: self.current_input_position(),
                    len: 1,
                });
                self.advance_input(1);
                continue;
            }
            self.error_input_at(
                format!("invalid input: {}", self.input[0..1].to_string()).as_str(),
            );
        }
        return tokens;
    }
}
