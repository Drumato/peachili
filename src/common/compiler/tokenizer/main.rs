use crate::common::{
    error::{CompileError as CE, TokenizeErrorKind as TEK},
    position::Position,
    token::{Token, TokenKind},
};

/// トークナイザのメインルーチン
pub fn main(source: String) -> Vec<Token> {
    tokenize(source)
}

fn tokenize(mut source: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    // トークンやエラーの位置用
    let mut row: usize = 1;
    let mut column: usize = 1;

    loop {
        let t = scan(&mut source, &mut row, &mut column);

        if let Err(e) = t {
            match e.get_kind() {
                // 単純にトークナイズ終了とする
                TEK::SOURCEISEMPTY => {
                    tokens.push(Token::new(TokenKind::EOF, Position::new(row, column)));
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

        if t.should_ignore() {
            continue;
        }

        tokens.push(t);
    }

    tokens
}

fn scan(source: &mut String, row: &mut usize, column: &mut usize) -> Result<Token, CE<TEK>> {
    if source.is_empty() {
        return Err(CE::new(
            TEK::SOURCEISEMPTY,
            Default::default(),
        ));
    }

    let cur_char = source.as_bytes()[0];

    match cur_char as char {
        // 文字列リテラル
        '"' => {
            let (t, len) = scan_string_literal(source, *row, *column);
            *column += len;
            source.drain(..len);
            Ok(t)
        }

        // 識別子 or キーワード
        c if c.is_ascii_alphabetic() => {
            let (t, len) = scan_identifier(source, *row, *column);
            *column += len;
            source.drain(..len);
            Ok(t)
        }

        // 整数/非符号付き整数
        number if number.is_ascii_digit() => match scan_number(source, *row, *column) {
            Ok((t, len)) => {
                *column += len;
                source.drain(..len);
                Ok(t)
            }
            Err(e) => {
                Err(e)
            }
        },

        // 空白類文字
        ' ' | '\t' => {
            let (t, len) = scan_whitespace(source, *row, *column);
            *column += len;
            source.drain(..len);
            Ok(t)
        }

        // 改行
        '\n' => {
            *row += 1;
            *column = 1;
            source.drain(..1);
            Ok(Token::new(TokenKind::NEWLINE, Position::new(0, 0)))
        }
        _ => {
            let (t, len) = scan_symbol(source, *row, *column);
            if let TokenKind::DOUBLESLASH = &t.get_kind() {
                let (t, len) = scan_comment(source, *row, *column);
                *column += len;
                source.drain(..len);
                return Ok(t);
            }

            source.drain(..len);
            Ok(t)
        }
    }
}

/// 文字列リテラルのトークン化
fn scan_string_literal(s: &str, row: usize, column: usize) -> (Token, usize) {
    let literal_pos = Position::new(row, column);

    // 文字列を切り取る
    let contents_str = cut_string_while(&s[1..], |c| c != &'"');
    let len = contents_str.len();

    // +2 -> 先頭/終端の `"` 分
    (
        Token::new_string_literal(contents_str, literal_pos),
        len + 2,
    )
}

/// 識別子 or 予約語
fn scan_identifier(s: &str, row: usize, column: usize) -> (Token, usize) {
    let ident_pos = Position::new(row, column);

    // 文字列を切り取る
    let ident_str = cut_string_while(s, |c| c.is_alphabetic() || c == &'_' || c.is_ascii_digit());
    let len = ident_str.len();

    let keyword = Token::try_new_keyword(&ident_str, ident_pos);
    if let Some(k) = keyword {
        return (k, len);
    }

    (Token::new_identifier(ident_str, ident_pos), len)
}

/// 記号
fn scan_symbol(s: &str, row: usize, column: usize) -> (Token, usize) {
    let symbol_pos = Position::new(row, column);
    let multilength_symbols = vec!["::", "//"];

    for sym_str in multilength_symbols.iter() {
        if s.starts_with(sym_str) {
            return (
                Token::new(TokenKind::new_symbol_from_str(sym_str), symbol_pos),
                2,
            );
        }
    }

    let symbols = vec![
        "+", "-", "*", "/", ";", "(", ")", "{", "}", "=", ",", "&", ".",
    ];

    for sym_str in symbols.iter() {
        if s.starts_with(sym_str) {
            return (
                Token::new(TokenKind::new_symbol_from_str(sym_str), symbol_pos),
                1,
            );
        }
    }

    unreachable!();
}

// コメント
fn scan_comment(s: &str, row: usize, column: usize) -> (Token, usize) {
    let comment_pos = Position::new(row, column);
    let comment_str = cut_string_while(s, |c| c != &'\n');
    let len = comment_str.len();

    (
        Token::new(
            TokenKind::COMMENT {
                contents: comment_str,
            },
            comment_pos,
        ),
        len,
    )
}

/// 整数/非符号付き整数のトークン化
fn scan_number(s: &str, row: usize, column: usize) -> Result<(Token, usize), CE<TEK>> {
    let literal_pos = Position::new(row, column);

    // 文字列を切り取る
    let number_str = cut_string_while(s, |c| c.is_ascii_digit());
    let len = number_str.len();

    let value = number_str.parse::<i64>();

    // 64bit整数として文字列を処理できなかった場合
    if value.is_err() {
        let err_pos = Position::new(row, column);
        return Err(CE::new(TEK::INTEGERLITERALOUTOFRANGE(number_str), err_pos));
    }

    // `100u` のようにuがついていればuint-literalとして処理
    if s.len() > len && s.as_bytes()[len] == b'u' {
        let u_value = number_str.parse::<u64>();

        return Ok((
            Token::new_uint_literal(u_value.unwrap(), literal_pos),
            len + 1,
        ));
    }

    Ok((Token::new_int_literal(value.unwrap(), literal_pos), len))
}

/// 空白類文字のトークン化
fn scan_whitespace(s: &str, row: usize, column: usize) -> (Token, usize) {
    let space_pos = Position::new(row, column);

    // 空白を切り取る
    let ws_str = cut_string_while(s, |c| c.is_whitespace() || c == &'\t');
    let length = ws_str.len();

    (Token::new_blank(space_pos), length)
}

/// 条件が通る間文字列を読み進め，切りとる
fn cut_string_while(s: &str, f: fn(&char) -> bool) -> String {
    s.chars().take_while(f).collect::<String>()
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    #[test]
    fn scan_number_test() {
        // 普通の場合
        let actual = scan_number("1000", 0, 0);
        assert!(actual.is_ok());

        let (t, len) = actual.unwrap();
        assert_eq!(4, len);

        assert_eq!(1000, t.int_value());

        let actual = scan_number("1000u", 0, 0);
        assert!(actual.is_ok());

        let (t, len) = actual.unwrap();
        assert_eq!(5, len);

        assert_eq!(1000, t.uint_value());
    }

    #[test]
    fn scan_identifier_test() {
        let (t, len) = scan_identifier("xyz", 0, 0);
        assert_eq!(3, len);
        assert_eq!("xyz", t.copy_name());

        let (t, len) = scan_identifier("ConstStr", 0, 0);
        assert_eq!(8, len);
        assert_eq!(&TokenKind::CONSTSTR, t.get_kind());
    }

    #[test]
    fn scan_string_literal_test() {
        let (t, len) = scan_string_literal("\"Drumato\"", 0, 0);
        assert_eq!(9, len);
        assert_eq!("Drumato", t.copy_contents());
    }

    #[test]
    fn scan_whitespace_test() {
        let (_, len) = scan_whitespace("1000", 0, 0);
        assert_eq!(0, len);

        let (_, len) = scan_whitespace("    1000", 0, 0);
        assert_eq!(4, len);

        let (_, len) = scan_whitespace("\t\t\t\t1000", 0, 0);
        assert_eq!(4, len);
    }

    #[test]
    fn scan_symbol_test() {
        let (t, len) = scan_symbol("+", 0, 0);
        assert_eq!(1, len);
        assert_eq!(&TokenKind::PLUS, t.get_kind());

        let (t, len) = scan_symbol("::", 0, 0);
        assert_eq!(2, len);
        assert_eq!(&TokenKind::DOUBLECOLON, t.get_kind());
    }

    #[test]
    fn scan_comment_test() {
        let (t, len) = scan_comment("// this is comment\n", 0, 0);
        assert_eq!(18, len);
        assert!(t.should_ignore());
    }

    #[test]
    fn scan_test() {
        let mut case = "100 200u \"String\"\nreturn_value ConstStr".to_string();
        let mut row = 1;
        let mut column = 1;

        // `100`
        let t = scan(&mut case, &mut row, &mut column);
        int_literal_test(t, 100, Position::new(1, 1));

        // ` `
        let t = scan(&mut case, &mut row, &mut column);
        ignore_test(t);

        // `200u`
        let t = scan(&mut case, &mut row, &mut column);
        uint_literal_test(t, 200, Position::new(1, 5));

        // ` `
        let t = scan(&mut case, &mut row, &mut column);
        ignore_test(t);

        // `"String"`
        let t = scan(&mut case, &mut row, &mut column);
        string_literal_test(t, "String", Position::new(1, 10));

        // `\n`
        let t = scan(&mut case, &mut row, &mut column);
        ignore_test(t);

        // `return_value`
        let t = scan(&mut case, &mut row, &mut column);
        identifier_test(t, "return_value", Position::new(2, 1));

        // ` `
        let t = scan(&mut case, &mut row, &mut column);
        ignore_test(t);

        // `ConstStr`
        let t = scan(&mut case, &mut row, &mut column);
        keyword_test(t, TokenKind::CONSTSTR, Position::new(2, 14));

        // EOF
        let t = scan(&mut case, &mut row, &mut column);
        assert!(t.is_err());
    }

    fn int_literal_test(t: Result<Token, CE<TEK>>, value: i64, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        let t_pos = t.get_position();
        assert_eq!(pos, t_pos);
        assert_eq!(value, t.int_value());
    }

    fn uint_literal_test(t: Result<Token, CE<TEK>>, value: u64, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        let t_pos = t.get_position();
        assert_eq!(pos, t_pos);
        assert_eq!(value, t.uint_value());
    }

    fn string_literal_test(t: Result<Token, CE<TEK>>, value: &str, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        let t_pos = t.get_position();
        assert_eq!(pos, t_pos);
        assert_eq!(value, t.copy_contents());
    }

    fn identifier_test(t: Result<Token, CE<TEK>>, name: &str, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        let t_pos = t.get_position();
        assert_eq!(pos, t_pos);
        assert_eq!(name, t.copy_name());
    }

    fn keyword_test(t: Result<Token, CE<TEK>>, k: TokenKind, pos: Position) {
        assert!(t.is_ok());

        let t = t.unwrap();
        let t_pos = t.get_position();
        assert_eq!(pos, t_pos);
        assert_eq!(&k, t.get_kind());
    }

    fn ignore_test(t: Result<Token, CE<TEK>>) {
        assert!(t.is_ok());
        assert!(t.unwrap().should_ignore())
    }
}
