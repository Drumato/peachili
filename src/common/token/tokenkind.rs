use std::fmt::{Display, Formatter, Result as FR};

/// トークンの種類
#[allow(dead_code)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum TokenKind {
    /// 整数リテラル( `100` )
    INTEGER { value: i64 },

    /// 非符号付き整数リテラル( `100u` )
    UNSIGNEDINTEGER { value: u64 },

    // TODO: あとでIDにするかも
    /// 文字列リテラル( `"Drumato"` )
    STRLIT { contents: String },

    /// 識別子( `drumato` )
    IDENTIFIER { name: String },

    // 記号
    /// `+`
    PLUS,
    /// `-`
    MINUS,
    /// `*`
    ASTERISK,
    /// `/`
    SLASH,
    /// `//`
    DOUBLESLASH,
    /// `&`
    AMPERSAND,
    /// `(`
    LPAREN,
    /// `)`
    RPAREN,
    /// `{`
    LBRACE,
    /// `}`
    RBRACE,
    /// `:`
    COLON,
    /// `::`
    DOUBLECOLON,
    /// `;`
    SEMICOLON,
    /// `=`
    ASSIGN,
    /// ` `
    BLANK,
    /// `\n`
    NEWLINE,
    /// `,`
    COMMA,
    /// `.`
    DOT,
    /// `(EOF)`
    EOF,
    /// `(COMMENT)`
    COMMENT { contents: String },

    // 予約語
    /// `asm`
    ASM,
    /// `begin`
    BEGIN,
    /// `Boolean`
    BOOLEAN,
    /// `const`
    CONST,
    /// `ConstStr`
    CONSTSTR,
    /// `countup`
    COUNTUP,
    /// `declare`
    DECLARE,
    /// `else`
    ELSE,
    /// `false`
    FALSE,
    /// `func`
    FUNC,
    /// `if`
    IF,
    /// `ifret`
    IFRET,
    /// `Int64`
    INT64,
    /// `import`
    IMPORT,
    /// `Noreturm`
    NORETURN,
    /// `pubtype`
    PUBTYPE,
    /// `return`
    RETURN,
    /// `asm`
    STRUCT,
    /// `true`
    TRUE,
    /// `Uint64`
    UINT64,
    /// `until`
    UNTIL,
    /// `varinit`
    VARINIT,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FR {
        let s = match self {
            TokenKind::INTEGER { value } => value.to_string(),

            TokenKind::UNSIGNEDINTEGER { value } => value.to_string(),

            TokenKind::STRLIT { contents } => format!("\"{}\"", contents),

            TokenKind::IDENTIFIER { name } => name.to_string(),

            // 記号
            TokenKind::PLUS => "+".to_string(),
            TokenKind::MINUS => "-".to_string(),
            TokenKind::ASTERISK => "*".to_string(),
            TokenKind::SLASH => "/".to_string(),
            TokenKind::DOUBLESLASH => "//".to_string(),
            TokenKind::AMPERSAND => "&".to_string(),
            TokenKind::LPAREN => "(".to_string(),
            TokenKind::RPAREN => ")".to_string(),
            TokenKind::LBRACE => "{".to_string(),
            TokenKind::RBRACE => "}".to_string(),
            TokenKind::COLON => ":".to_string(),
            TokenKind::DOUBLECOLON => "::".to_string(),
            TokenKind::SEMICOLON => ";".to_string(),
            TokenKind::ASSIGN => "=".to_string(),
            TokenKind::BLANK => "(BLANK)".to_string(),
            TokenKind::NEWLINE => "(NEWLINE)".to_string(),
            TokenKind::COMMA => ",".to_string(),
            TokenKind::DOT => ".".to_string(),
            TokenKind::EOF => "(EOF)".to_string(),
            TokenKind::COMMENT { contents: _ } => "(COMMENT)".to_string(),

            // 予約語
            TokenKind::ASM => "asm".to_string(),
            TokenKind::BEGIN => "begin".to_string(),
            TokenKind::BOOLEAN => "Boolean".to_string(),
            TokenKind::CONST => "const".to_string(),
            TokenKind::CONSTSTR => "ConstStr".to_string(),
            TokenKind::COUNTUP => "countup".to_string(),
            TokenKind::DECLARE => "declare".to_string(),
            TokenKind::ELSE => "else".to_string(),
            TokenKind::FALSE => "false".to_string(),
            TokenKind::FUNC => "func".to_string(),
            TokenKind::IF => "if".to_string(),
            TokenKind::IFRET => "ifret".to_string(),
            TokenKind::IMPORT => "import".to_string(),
            TokenKind::INT64 => "Int64".to_string(),
            TokenKind::NORETURN => "Noreturn".to_string(),
            TokenKind::PUBTYPE => "pubtype".to_string(),
            TokenKind::RETURN => "return".to_string(),
            TokenKind::STRUCT => "struct".to_string(),
            TokenKind::TRUE => "true".to_string(),
            TokenKind::UINT64 => "Uint64".to_string(),
            TokenKind::UNTIL => "until".to_string(),
            TokenKind::VARINIT => "varinit".to_string(),
        };

        write!(f, "{}", s)
    }
}

impl TokenKind {
    pub fn new_symbol_from_str(s: &str) -> Self {
        match s {
            "+" => TokenKind::PLUS,
            "-" => TokenKind::MINUS,
            "*" => TokenKind::ASTERISK,
            "/" => TokenKind::SLASH,
            "//" => TokenKind::DOUBLESLASH,
            "(" => TokenKind::LPAREN,
            ")" => TokenKind::RPAREN,
            "{" => TokenKind::LBRACE,
            "}" => TokenKind::RBRACE,
            ":" => TokenKind::COLON,
            "::" => TokenKind::DOUBLECOLON,
            "=" => TokenKind::ASSIGN,
            "," => TokenKind::COMMA,
            ";" => TokenKind::SEMICOLON,
            "&" => TokenKind::AMPERSAND,
            "." => TokenKind::DOT,
            _ => panic!("invalid tokenkind from {}", s),
        }
    }
    pub fn try_new_keyword(s: &str) -> Option<Self> {
        match s {
            "asm" => Some(TokenKind::ASM),
            "begin" => Some(TokenKind::BEGIN),
            "Boolean" => Some(TokenKind::BOOLEAN),
            "const" => Some(TokenKind::CONST),
            "ConstStr" => Some(TokenKind::CONSTSTR),
            "countup" => Some(TokenKind::COUNTUP),
            "declare" => Some(TokenKind::DECLARE),
            "else" => Some(TokenKind::ELSE),
            "false" => Some(TokenKind::FALSE),
            "func" => Some(TokenKind::FUNC),
            "if" => Some(TokenKind::IF),
            "ifret" => Some(TokenKind::IFRET),
            "import" => Some(TokenKind::IMPORT),
            "Int64" => Some(TokenKind::INT64),
            "Noreturn" => Some(TokenKind::NORETURN),
            "pubtype" => Some(TokenKind::PUBTYPE),
            "return" => Some(TokenKind::RETURN),
            "struct" => Some(TokenKind::STRUCT),
            "true" => Some(TokenKind::TRUE),
            "Uint64" => Some(TokenKind::UINT64),
            "until" => Some(TokenKind::UNTIL),
            "varinit" => Some(TokenKind::VARINIT),
            _ => None,
        }
    }
}
