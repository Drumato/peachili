use crate::common::{
    error::{CompileError as CE, TokenizeErrorKind as TEK},
    position::Position,
    token::{Token, TokenKind},
};

/// トークナイザのメインルーチン
pub fn main(source: String) -> Vec<Token> {
    tokenize(source)
}

/// トークンに与える情報等を集約
/// トークン列自体をもたせると読みづらくなるので持たせない
struct Tokenization {
    row: usize,
    column: usize,
    /// 現在作成しているトークンが該当する文字列の長さ
    cur_token_length: usize,
}

fn tokenize(mut source: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    // トークンやエラーの位置用
    let mut tokenization = Tokenization {
        row: 1,
        column: 1,
        cur_token_length: 0,
    };

    loop {
        let t = tokenization.scan(&mut source);

        if let Err(e) = t {
            match e.get_kind() {
                // 単純にトークナイズ終了とする
                TEK::SOURCEISEMPTY => {
                    push_eof_token(&mut tokens, tokenization.row, tokenization.column);
                    break;
                }
                // 字句解析エラーなので，出力して終了
                _ => {
                    e.output();
                    std::process::exit(1);
                }
            }
        }

        let t = t.unwrap();

        // 空白類文字は読み飛ばす
        if t.should_ignore() {
            continue;
        }

        tokens.push(t);
    }

    tokens
}

impl Tokenization {
    /// 文字列の先頭を見て，字句規則を適用する
    fn scan(&mut self, source: &mut String) -> Result<Token, CE<TEK>> {
        if source.is_empty() {
            return Err(CE::new(TEK::SOURCEISEMPTY, Default::default()));
        }

        let cur_char = source.as_bytes()[0];

        match cur_char as char {
            // 文字列リテラル
            '"' => {
                let t = self.scan_string_literal(source);
                source.drain(..self.cur_token_length);
                Ok(t)
            }

            // 識別子 or キーワード
            c if c.is_ascii_alphabetic() => {
                let t = self.scan_identifier(source);
                source.drain(..self.cur_token_length);
                Ok(t)
            }

            // 整数/非符号付き整数
            number if number.is_ascii_digit() => match self.scan_number(source) {
                Ok(t) => {
                    source.drain(..self.cur_token_length);
                    Ok(t)
                }
                Err(e) => Err(e),
            },

            // 空白類文字
            ' ' | '\t' => {
                let t = self.scan_whitespace(source);
                source.drain(..self.cur_token_length);
                Ok(t)
            }

            // 改行
            '\n' => {
                self.row += 1;
                self.column = 1;
                source.drain(..1);
                Ok(Token::new(TokenKind::NEWLINE, Position::new(0, 0)))
            }

            // コメントまたは記号とする
            _ => {
                let t = self.scan_symbol(source);
                if let TokenKind::DOUBLESLASH = &t.get_kind() {
                    let t = self.scan_comment(source);
                    source.drain(..self.cur_token_length);
                    return Ok(t);
                }

                source.drain(..self.cur_token_length);
                Ok(t)
            }
        }
    }

    /// 文字列リテラルのトークン化
    fn scan_string_literal(&mut self, s: &str) -> Token {
        let literal_pos = Position::new(self.row, self.column);

        // 文字列を切り取る
        let contents_str = cut_string_while(&s[1..], |c| c != &'"');
        // +2 -> 先頭/終端の `"` 分
        let len = contents_str.len() + 2;

        self.condition_position(len);

        Token::new_string_literal(contents_str, literal_pos)
    }

    /// 識別子 or 予約語
    fn scan_identifier(&mut self, s: &str) -> Token {
        let ident_pos = Position::new(self.row, self.column);

        // 文字列を切り取る
        let ident_str =
            cut_string_while(s, |c| c.is_alphabetic() || c == &'_' || c.is_ascii_digit());
        let len = ident_str.len();
        self.condition_position(len);

        let keyword = Token::try_new_keyword(&ident_str, ident_pos);
        if let Some(k) = keyword {
            return k;
        }

        Token::new_identifier(ident_str, ident_pos)
    }

    /// 記号
    fn scan_symbol(&mut self, s: &str) -> Token {
        let symbol_pos = Position::new(self.row, self.column);
        let symbol_str = s[..2].to_string();

        let symbol_kind = match symbol_str.as_str() {
            "->" | "::" | "//" => {
                self.condition_position(2);
                TokenKind::new_symbol_from_str(&symbol_str)
            }

            // '()' などに対応するため，ネストしたmatch式を用いる
            _ => {
                let symbol_str = symbol_str.as_bytes()[0];

                match symbol_str as char {
                    '+' | '-' | '*' | '/' | ':' | ';' | '(' | ')' | '{' | '}' | '=' | ',' | '&'
                    | '.' => {
                        self.condition_position(1);
                        TokenKind::new_symbol_from_str(&(symbol_str as char).to_string())
                    }
                    _ => panic!("undefined such an symbol => '{}'", symbol_str as char),
                }
            }
        };

        Token::new(symbol_kind, symbol_pos)
    }

    /// コメント
    fn scan_comment(&mut self, s: &str) -> Token {
        let comment_pos = Position::new(self.row, self.column);
        let comment_str = cut_string_while(s, |c| c != &'\n');
        let len = comment_str.len();
        self.condition_position(len);

        Token::new(
            TokenKind::COMMENT {
                contents: comment_str,
            },
            comment_pos,
        )
    }

    /// 整数/非符号付き整数のトークン化
    fn scan_number(&mut self, s: &str) -> Result<Token, CE<TEK>> {
        let literal_pos = Position::new(self.row, self.column);

        // 文字列を切り取る
        let number_str = cut_string_while(s, |c| c.is_ascii_digit());
        let len = number_str.len();
        self.condition_position(len);

        let value = number_str.parse::<i64>();

        // 64bit整数として文字列を処理できなかった場合
        if value.is_err() {
            let err_pos = Position::new(self.row, self.column);
            return Err(CE::new(TEK::INTEGERLITERALOUTOFRANGE(number_str), err_pos));
        }

        // `100u` のようにuがついていればuint-literalとして処理
        if s.len() > len && s.as_bytes()[len] == b'u' {
            self.column += 1;
            self.cur_token_length += 1;
            let u_value = number_str.parse::<u64>();

            return Ok(Token::new_uint_literal(u_value.unwrap(), literal_pos));
        }

        Ok(Token::new_int_literal(value.unwrap(), literal_pos))
    }

    /// 空白類文字のトークン化
    fn scan_whitespace(&mut self, s: &str) -> Token {
        let space_pos = Position::new(self.row, self.column);

        // 空白を切り取る
        let ws_str = cut_string_while(s, |c| c.is_whitespace() || c == &'\t');
        let length = ws_str.len();
        self.condition_position(length);

        Token::new_blank(space_pos)
    }

    fn condition_position(&mut self, length: usize) {
        self.column += length;
        self.cur_token_length = length;
    }
}

/// 条件が通る間文字列を読み進め，切りとる
fn cut_string_while(s: &str, f: fn(&char) -> bool) -> String {
    s.chars().take_while(f).collect::<String>()
}

fn push_eof_token(tokens: &mut Vec<Token>, row: usize, column: usize) {
    let eof_pos = Position::new(row, column);
    tokens.push(Token::new(TokenKind::EOF, eof_pos));
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    #[test]
    fn scan_number_test() {
        let mut tokenization = new_tokenization();

        // 普通の場合
        let actual = tokenization.scan_number("1000");
        int_literal_helper(actual, 1000, Position::new(1, 1));

        let actual = tokenization.scan_number("1000u");
        uint_literal_helper(actual, 1000, Position::new(1, 5))
    }

    #[test]
    fn scan_identifier_test() {
        let mut tokenization = new_tokenization();
        let t = tokenization.scan_identifier("xyz");
        identifier_helper(
            t,
            TokenKind::IDENTIFIER {
                name: "xyz".to_string(),
            },
            Position::new(1, 1),
        );

        let t = tokenization.scan_identifier("ConstStr");
        identifier_helper(t, TokenKind::CONSTSTR, Position::new(1, 4));
    }

    #[test]
    fn scan_string_literal_test() {
        let mut tokenization = new_tokenization();
        let t = tokenization.scan_string_literal("\"Drum\"");
        string_literal_helper(t, "Drum", Position::new(1, 1));
    }

    #[test]
    fn scan_whitespace_test() {
        let mut tokenization = new_tokenization();
        tokenization.scan_whitespace("1000");
        assert_eq!(0, tokenization.cur_token_length);

        tokenization.scan_whitespace("    1000");
        assert_eq!(4, tokenization.cur_token_length);

        tokenization.scan_whitespace("\t\t\t\t1000");
        assert_eq!(4, tokenization.cur_token_length);
    }

    #[test]
    fn scan_symbol_test() {
        let mut tokenization = new_tokenization();
        let t = tokenization.scan_symbol("+ ");
        symbol_helper(t, TokenKind::PLUS, Position::new(1, 1));

        let t = tokenization.scan_symbol("::");
        symbol_helper(t, TokenKind::DOUBLECOLON, Position::new(1, 2));
    }

    #[test]
    fn scan_comment_test() {
        let mut tokenization = new_tokenization();
        let t = tokenization.scan_comment("// this is comment\n");
        assert!(t.should_ignore());
        assert_eq!(18, tokenization.cur_token_length);
    }

    #[test]
    fn scan_test() {
        let mut case = "100 200u \"String\"\nreturn_value ConstStr".to_string();
        let mut tokenization = new_tokenization();

        // `100`
        let t = tokenization.scan(&mut case);
        int_literal_helper(t, 100, Position::new(1, 1));

        // ` `
        let t = tokenization.scan(&mut case);
        ignore_helper(t);

        // `200u`
        let t = tokenization.scan(&mut case);
        uint_literal_helper(t, 200, Position::new(1, 5));

        // ` `
        let t = tokenization.scan(&mut case);
        ignore_helper(t);

        // `"String"`
        let t = tokenization.scan(&mut case);
        string_literal_helper(t.unwrap(), "String", Position::new(1, 10));

        // `\n`
        let t = tokenization.scan(&mut case);
        ignore_helper(t);

        // `return_value`
        let t = tokenization.scan(&mut case);
        identifier_helper(
            t.unwrap(),
            TokenKind::IDENTIFIER {
                name: "return_value".to_string(),
            },
            Position::new(2, 1),
        );

        // ` `
        let t = tokenization.scan(&mut case);
        ignore_helper(t);

        // `ConstStr`
        let t = tokenization.scan(&mut case);
        identifier_helper(t.unwrap(), TokenKind::CONSTSTR, Position::new(2, 14));

        // EOF
        let t = tokenization.scan(&mut case);
        assert!(t.is_err());
    }

    fn int_literal_helper(t: Result<Token, CE<TEK>>, value: i64, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        assert_eq!(pos, t.get_position());
        assert_eq!(value, t.int_value());
    }

    fn uint_literal_helper(t: Result<Token, CE<TEK>>, value: u64, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        assert_eq!(pos, t.get_position());
        assert_eq!(value, t.uint_value());
    }

    fn identifier_helper(t: Token, k: TokenKind, pos: Position) {
        assert_eq!(&k, t.get_kind());
        assert_eq!(pos, t.get_position());
    }

    fn string_literal_helper(t: Token, value: &str, pos: Position) {
        assert_eq!(pos, t.get_position());
        assert_eq!(value, t.copy_contents());
    }

    fn symbol_helper(t: Token, k: TokenKind, pos: Position) {
        assert_eq!(&k, t.get_kind());
        assert_eq!(pos, t.get_position());
    }

    fn ignore_helper(t: Result<Token, CE<TEK>>) {
        assert!(t.is_ok());
        assert!(t.unwrap().should_ignore())
    }

    fn new_tokenization() -> Tokenization {
        Tokenization {
            row: 1,
            column: 1,
            cur_token_length: 0,
        }
    }
}
