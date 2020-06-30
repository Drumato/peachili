use crate::common::position as pos;
use crate::compiler::general::resource as res;

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    position: pos::Position,
}

impl Token {
    pub fn new(cur_pos: pos::Position, token_kind: TokenKind) -> Self {
        Self {
            kind: token_kind,
            position: cur_pos,
        }
    }
    pub fn should_ignore(&self) -> bool {
        match self.kind {
            TokenKind::BLANK | TokenKind::NEWLINE => true,
            _ => false,
        }
    }

    pub fn new_int(cur_pos: pos::Position, int_value: i64) -> Self {
        Self::new(cur_pos, TokenKind::INTEGER(int_value))
    }
    pub fn new_uint(cur_pos: pos::Position, int_value: u64) -> Self {
        Self::new(cur_pos, TokenKind::UNSIGNEDINTEGER(int_value))
    }
    pub fn get_pos(&self) -> pos::Position {
        let (row, column) = self.position.get_pos();
        pos::Position::new(row, column)
    }

    pub fn get_int_value(&self) -> Option<i64> {
        match self.kind {
            TokenKind::INTEGER(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_uint_value(&self) -> Option<u64> {
        match self.kind {
            TokenKind::UNSIGNEDINTEGER(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_ident_id(&self) -> Option<res::PStringId> {
        match &self.kind {
            TokenKind::IDENTIFIER(name_id) => Some(*name_id),
            _ => None,
        }
    }

    pub fn get_str_id(&self) -> Option<res::PStringId> {
        match &self.kind {
            TokenKind::STRLIT(contents_id) => Some(*contents_id),
            _ => None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.kind)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    INTEGER(i64),
    UNSIGNEDINTEGER(u64),
    STRLIT(res::PStringId),
    IDENTIFIER(res::PStringId),

    // 記号
    PLUS,
    MINUS,
    ASTERISK,
    SLASH,
    DOUBLESLASH,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    COLON,
    DOUBLECOLON,
    SEMICOLON,
    ASSIGN,
    BLANK,
    NEWLINE,
    COMMA,
    EOF,
    AMPERSAND,

    // 予約語
    ASM,
    BOOLEAN,
    CONST,
    CONSTSTR,
    COUNTUP,
    DECLARE,
    ELSE,
    FALSE,
    FROM,
    FUNC,
    IF,
    IFRET,
    INT64,
    NORETURN,
    PUBTYPE,
    RETURN,
    REQUIRE,
    STRUCT,
    TO,
    TRUE,
    UINT64,
    VARINIT,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenKind::INTEGER(val) => write!(f, "{}", val),
            TokenKind::UNSIGNEDINTEGER(val) => write!(f, "{}u", val),
            TokenKind::STRLIT(id) => write!(f, "str-{:?}", id),
            TokenKind::IDENTIFIER(id) => write!(f, "ident-{:?}", id),

            // 記号
            TokenKind::PLUS => write!(f, "+"),
            TokenKind::MINUS => write!(f, "-"),
            TokenKind::ASTERISK => write!(f, "*"),
            TokenKind::SLASH => write!(f, "/"),
            TokenKind::DOUBLESLASH => write!(f, "//"),
            TokenKind::LPAREN => write!(f, "("),
            TokenKind::RPAREN => write!(f, ")"),
            TokenKind::LBRACE => write!(f, "{{"),
            TokenKind::RBRACE => write!(f, "}}"),
            TokenKind::COLON => write!(f, ":"),
            TokenKind::DOUBLECOLON => write!(f, "::"),
            TokenKind::SEMICOLON => write!(f, ";"),
            TokenKind::ASSIGN => write!(f, "="),
            TokenKind::AMPERSAND => write!(f, "&"),
            TokenKind::BLANK => write!(f, "BLANK"),
            TokenKind::NEWLINE => write!(f, "NEWLINE"),
            TokenKind::COMMA => write!(f, ","),
            TokenKind::EOF => write!(f, "eof"),

            // 予約語
            TokenKind::STRUCT => write!(f, "struct"),
            TokenKind::BOOLEAN => write!(f, "Boolean"),
            TokenKind::TRUE => write!(f, "true"),
            TokenKind::FALSE => write!(f, "false"),
            TokenKind::REQUIRE => write!(f, "require"),
            TokenKind::IF => write!(f, "if"),
            TokenKind::ELSE => write!(f, "else"),
            TokenKind::IFRET => write!(f, "ifret"),
            TokenKind::FUNC => write!(f, "func"),
            TokenKind::DECLARE => write!(f, "declare"),
            TokenKind::COUNTUP => write!(f, "countup"),
            TokenKind::FROM => write!(f, "from"),
            TokenKind::TO => write!(f, "to"),
            TokenKind::ASM => write!(f, "asm"),
            TokenKind::RETURN => write!(f, "return"),
            TokenKind::NORETURN => write!(f, "Noreturn"),
            TokenKind::INT64 => write!(f, "Int64"),
            TokenKind::UINT64 => write!(f, "Uint64"),
            TokenKind::CONSTSTR => write!(f, "ConstStr"),
            TokenKind::PUBTYPE => write!(f, "pubtype"),
            TokenKind::VARINIT => write!(f, "varinit"),
            TokenKind::CONST => write!(f, "const"),
        }
    }
}

impl TokenKind {
    pub fn new_from_string(s: &str) -> Self {
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
            _ => panic!("invalid tokenkind from {}", s),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            TokenKind::INTEGER(_val) => "integer",
            TokenKind::UNSIGNEDINTEGER(_val) => "unsigned-integer",
            TokenKind::STRLIT(_val) => "string-lit",
            TokenKind::IDENTIFIER(_val) => "identifier",

            // 記号
            TokenKind::PLUS => "+",
            TokenKind::MINUS => "-",
            TokenKind::ASTERISK => "*",
            TokenKind::SLASH => "/",
            TokenKind::DOUBLESLASH => "//",
            TokenKind::LPAREN => "(",
            TokenKind::RPAREN => ")",
            TokenKind::LBRACE => "{",
            TokenKind::RBRACE => "}",
            TokenKind::COLON => ":",
            TokenKind::DOUBLECOLON => "::",
            TokenKind::SEMICOLON => ";",
            TokenKind::ASSIGN => "=",
            TokenKind::BLANK => "blank",
            TokenKind::NEWLINE => "newline",
            TokenKind::COMMA => ",",
            TokenKind::EOF => "eof",
            TokenKind::AMPERSAND => "&",

            // 予約語
            TokenKind::STRUCT => "strct",
            TokenKind::BOOLEAN => "boolean",
            TokenKind::TRUE => "true",
            TokenKind::FALSE => "false",
            TokenKind::REQUIRE => "require",
            TokenKind::IF => "if",
            TokenKind::ELSE => "else",
            TokenKind::IFRET => "ifret",
            TokenKind::FUNC => "func",
            TokenKind::DECLARE => "declare",
            TokenKind::COUNTUP => "countup",
            TokenKind::FROM => "from",
            TokenKind::TO => "to",
            TokenKind::ASM => "asm",
            TokenKind::RETURN => "return",
            TokenKind::NORETURN => "Noreturn",
            TokenKind::INT64 => "Int64",
            TokenKind::UINT64 => "Uint64",
            TokenKind::CONSTSTR => "ConstStr",
            TokenKind::PUBTYPE => "pubtype",
            TokenKind::VARINIT => "varinit",
            TokenKind::CONST => "const",
        }
    }
}
