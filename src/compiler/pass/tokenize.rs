use crate::common::option;
use crate::compiler::resource as res;

pub fn tokenize(opt: &option::BuildOption, contents: String) -> Vec<res::Token> {
    let mut lexer = res::Lexer::new(opt, contents);

    lexer.construct_tokens();

    // lexer is dropped after calling `lexer.give_token()`.
    lexer.give_token()
}

impl<'a> res::Lexer<'a> {
    fn construct_tokens(&mut self) {
        while let Some(t) = self.scan() {
            if t.should_ignore() {
                continue;
            }

            self.add_token(t);

            if let res::TokenKind::EOF = self.last_token().kind {
                break;
            }
        }
    }

    fn scan(&mut self) -> Option<res::Token> {
        if self.offset_overruns_contents_length() {
            let cur_pos = self.cur_position();
            return Some(res::Token::new(cur_pos, res::TokenKind::EOF));
        }

        let look_char = self.cur_offset_char();
        match look_char {
            // 整数トークン
            number if number.is_ascii_digit() => self.scan_number(),

            // 文字列リテラル
            '"' => self.scan_strlit(),

            // 識別子
            c if c.is_ascii_alphabetic() => self.scan_word(),
            '_' => self.scan_word(),

            // 空白類文字
            ' ' | '\t' => self.skip_whitespace(),
            '\n' => {
                let cur_pos = self.cur_position();
                self.skip_crlf();
                Some(res::Token::new(cur_pos, res::TokenKind::NEWLINE))
            }

            _ => self.scan_symbol(),
        }
    }

    fn scan_strlit(&mut self) -> Option<res::Token> {
        let cur_pos = self.cur_position();

        self.skip_offset(1);

        let contents = self.cut_contents(|c| c != &'"');

        // 終端の `"` を読み飛ばすため+1
        self.skip_offset(contents.len() + 1);

        Some(res::Token::new(cur_pos, res::TokenKind::STRLIT(contents)))
    }
    fn scan_word(&mut self) -> Option<res::Token> {
        // 現在のオフセットを退避
        let cur_pos = self.cur_position();

        // 文字列を読み取る
        let word = self.cut_contents(|c| c.is_alphabetic() || c == &'_' || c.is_ascii_digit());

        // オフセットを進める
        self.skip_offset(word.len());

        // 予約語かチェック
        if let Some(t_kind) = self.check_reserved(&word) {
            return Some(res::Token::new(cur_pos, t_kind));
        }

        // 識別子
        Some(res::Token::new(cur_pos, res::TokenKind::IDENTIFIER(word)))
    }

    fn scan_number(&mut self) -> Option<res::Token> {
        let number_str = self.cut_contents(|c| c.is_ascii_digit());
        let length = number_str.len();
        let decimal_value = number_str.parse::<i64>();

        if decimal_value.is_err() {
            panic!("{} can't consider to int-value", &number_str);
        }

        let cur_pos = self.cur_position();
        self.skip_offset(length);

        Some(res::Token::new_int(cur_pos, decimal_value.unwrap()))
    }

    fn scan_symbol(&mut self) -> Option<res::Token> {
        let multilength_symbols = vec!["::"];

        for sym_str in multilength_symbols.iter() {
            if self.contents_starts_with(sym_str) {
                let length = sym_str.len();
                let cur_pos = self.cur_position();
                self.skip_offset(length);

                return Some(res::Token::new(cur_pos, res::TokenKind::from_str(sym_str)));
            }
        }

        let simple_symbols = vec!["+", "-", "*", "/", ";", "(", ")", "{", "}", "=", ","];

        for sym_str in simple_symbols.iter() {
            if self.contents_starts_with(sym_str) {
                let cur_pos = self.cur_position();
                self.skip_offset(1);

                return Some(res::Token::new(cur_pos, res::TokenKind::from_str(sym_str)));
            }
        }

        None
    }

    fn skip_whitespace(&mut self) -> Option<res::Token> {
        let cur_pos = self.cur_position();
        let ws = self.cut_contents(|c| c.is_whitespace() || c == &'\t');
        self.skip_offset(ws.len());

        Some(res::Token::new(cur_pos, res::TokenKind::BLANK))
    }

    fn check_reserved(&self, s: &str) -> Option<res::TokenKind> {
        match s {
            "ifret" => Some(res::TokenKind::IFRET),
            "if" => Some(res::TokenKind::IF),
            "else" => Some(res::TokenKind::ELSE),
            "int64" => Some(res::TokenKind::INT64),
            "func" => Some(res::TokenKind::FUNC),
            "return" => Some(res::TokenKind::RETURN),
            "declare" => Some(res::TokenKind::DECLARE),
            "countup" => Some(res::TokenKind::COUNTUP),
            "from" => Some(res::TokenKind::FROM),
            "to" => Some(res::TokenKind::TO),
            "require" => Some(res::TokenKind::REQUIRE),
            "asm" => Some(res::TokenKind::ASM),
            "noreturn" => Some(res::TokenKind::NORETURN),
            "str" => Some(res::TokenKind::STR),
            _ => None,
        }
    }
}
