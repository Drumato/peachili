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

    // 予約語
    TRUE,
    FALSE,
    BOOLEAN,
    REQUIRE,
    IF,
    ELSE,
    IFRET,
    FUNC,
    DECLARE,
    COUNTUP,
    FROM,
    TO,
    ASM,
    RETURN,
    NORETURN,
    INT64,
    UINT64,
    STR,
    PUBTYPE,
    VARINIT,
    CONST,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::INTEGER(val) => write!(f, "{}", val),
            Self::UNSIGNEDINTEGER(val) => write!(f, "{}u", val),
            Self::STRLIT(id) => write!(f, "str-{:?}", id),
            Self::IDENTIFIER(id) => write!(f, "ident-{:?}", id),

            // 記号
            Self::PLUS => write!(f, "+"),
            Self::MINUS => write!(f, "-"),
            Self::ASTERISK => write!(f, "*"),
            Self::SLASH => write!(f, "/"),
            Self::DOUBLESLASH => write!(f, "//"),
            Self::LPAREN => write!(f, "("),
            Self::RPAREN => write!(f, ")"),
            Self::LBRACE => write!(f, "{{"),
            Self::RBRACE => write!(f, "}}"),
            Self::COLON => write!(f, ":"),
            Self::DOUBLECOLON => write!(f, "::"),
            Self::SEMICOLON => write!(f, ";"),
            Self::ASSIGN => write!(f, "="),
            Self::BLANK => write!(f, "BLANK"),
            Self::NEWLINE => write!(f, "NEWLINE"),
            Self::COMMA => write!(f, ","),
            Self::EOF => write!(f, "eof"),

            // 予約語
            Self::BOOLEAN => write!(f, "boolean"),
            Self::TRUE => write!(f, "true"),
            Self::FALSE => write!(f, "false"),
            Self::REQUIRE => write!(f, "require"),
            Self::IF => write!(f, "if"),
            Self::ELSE => write!(f, "else"),
            Self::IFRET => write!(f, "ifret"),
            Self::FUNC => write!(f, "func"),
            Self::DECLARE => write!(f, "declare"),
            Self::COUNTUP => write!(f, "countup"),
            Self::FROM => write!(f, "from"),
            Self::TO => write!(f, "to"),
            Self::ASM => write!(f, "asm"),
            Self::RETURN => write!(f, "return"),
            Self::NORETURN => write!(f, "noreturn"),
            Self::INT64 => write!(f, "int64"),
            Self::UINT64 => write!(f, "uint64"),
            Self::STR => write!(f, "str"),
            Self::PUBTYPE => write!(f, "pubtype"),
            Self::VARINIT => write!(f, "varinit"),
            Self::CONST => write!(f, "const"),
        }
    }
}

impl TokenKind {
    pub fn new_from_string(s: &str) -> Self {
        match s {
            "+" => Self::PLUS,
            "-" => Self::MINUS,
            "*" => Self::ASTERISK,
            "/" => Self::SLASH,
            "//" => Self::DOUBLESLASH,
            "(" => Self::LPAREN,
            ")" => Self::RPAREN,
            "{" => Self::LBRACE,
            "}" => Self::RBRACE,
            ":" => Self::COLON,
            "::" => Self::DOUBLECOLON,
            "=" => Self::ASSIGN,
            "," => Self::COMMA,
            ";" => Self::SEMICOLON,
            _ => panic!("invalid tokenkind from {}", s),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::INTEGER(_val) => "integer",
            Self::UNSIGNEDINTEGER(_val) => "unsigned-integer",
            Self::STRLIT(_val) => "string-lit",
            Self::IDENTIFIER(_val) => "identifier",

            // 記号
            Self::PLUS => "+",
            Self::MINUS => "-",
            Self::ASTERISK => "*",
            Self::SLASH => "/",
            Self::DOUBLESLASH => "//",
            Self::LPAREN => "(",
            Self::RPAREN => ")",
            Self::LBRACE => "{",
            Self::RBRACE => "}",
            Self::COLON => ":",
            Self::DOUBLECOLON => "::",
            Self::SEMICOLON => ";",
            Self::ASSIGN => "=",
            Self::BLANK => "blank",
            Self::NEWLINE => "newline",
            Self::COMMA => ",",
            Self::EOF => "eof",

            // 予約語
            Self::BOOLEAN => "boolean",
            Self::TRUE => "true",
            Self::FALSE => "false",
            Self::REQUIRE => "require",
            Self::IF => "if",
            Self::ELSE => "else",
            Self::IFRET => "ifret",
            Self::FUNC => "func",
            Self::DECLARE => "declare",
            Self::COUNTUP => "countup",
            Self::FROM => "from",
            Self::TO => "to",
            Self::ASM => "asm",
            Self::RETURN => "return",
            Self::NORETURN => "noreturn",
            Self::INT64 => "int64",
            Self::UINT64 => "uint64",
            Self::STR => "str",
            Self::PUBTYPE => "pubtype",
            Self::VARINIT => "varinit",
            Self::CONST => "const",
        }
    }
}
