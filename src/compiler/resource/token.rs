use crate::common::position as pos;

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

    pub fn new_int(cur_pos: pos::Position, int_value: i128) -> Self {
        Self::new(cur_pos, TokenKind::INTEGER(int_value))
    }
    pub fn get_pos(&self) -> pos::Position {
        let (row, column) = self.position.get_pos();
        pos::Position::new(row, column)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.kind.to_str())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdentName {
    name: String,
    next: Option<Box<IdentName>>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    INTEGER(i128),
    STRLIT(String),
    IDENTIFIER(String),

    // 記号
    PLUS,
    MINUS,
    ASTERISK,
    SLASH,
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
    INT,
    STR,
}

impl TokenKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "+" => Self::PLUS,
            "-" => Self::MINUS,
            "*" => Self::ASTERISK,
            "/" => Self::SLASH,
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
            Self::STRLIT(_val) => "string-lit",
            Self::IDENTIFIER(_val) => "identifier",

            // 記号
            Self::PLUS => "+",
            Self::MINUS => "-",
            Self::ASTERISK => "*",
            Self::SLASH => "/",
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
            Self::INT => "int",
            Self::STR => "str",
        }
    }
}
