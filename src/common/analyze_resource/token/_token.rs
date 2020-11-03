use crate::common::{position::Position, token::TokenKind};
use std::fmt::{Display, Formatter, Result as FR};

/// トークナイザが返すリストの各要素
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Token {
    /// トークンの種類
    k: TokenKind,
    /// ファイル上の位置
    p: Position,
}

#[allow(dead_code)]
impl Token {
    pub fn new(kind: TokenKind, position: Position) -> Self {
        Self {
            k: kind,
            p: position,
        }
    }

    pub fn try_new_keyword(s: &str, p: Position) -> Option<Self> {
        let tk = TokenKind::try_new_keyword(s);
        tk.as_ref()?;

        Some(Token::new(tk.unwrap(), p))
    }

    pub fn get_position(&self) -> Position {
        self.p
    }

    /// 識別子
    pub fn new_identifier(name: String, position: Position) -> Self {
        Self::new(TokenKind::IDENTIFIER { name }, position)
    }

    /// 文字列トークンの定義
    pub fn new_string_literal(contents: String, position: Position) -> Self {
        Self::new(TokenKind::STRLIT { contents }, position)
    }

    /// 整数トークンの定義
    pub fn new_int_literal(v: i64, position: Position) -> Self {
        Self::new(TokenKind::Integer { value: v }, position)
    }

    /// 非符号付き整数トークンの定義
    pub fn new_uint_literal(v: u64, position: Position) -> Self {
        Self::new(TokenKind::UNSIGNEDINTEGER { value: v }, position)
    }

    /// 空白の定義
    pub fn new_blank(position: Position) -> Self {
        Self::new(TokenKind::BLANK, position)
    }

    /// 整数トークンだと仮定して，数値を受け取る
    pub fn int_value(&self) -> i64 {
        match &self.k {
            TokenKind::Integer { value } => *value,
            _ => unimplemented!(),
        }
    }

    /// 非符号付き整数トークンだと仮定して，数値を受け取る
    pub fn uint_value(&self) -> u64 {
        match &self.k {
            TokenKind::UNSIGNEDINTEGER { value } => *value,
            _ => unimplemented!(),
        }
    }

    /// 文字列トークンだと仮定して，文字列を受け取る
    pub fn copy_contents(&self) -> String {
        match &self.k {
            TokenKind::STRLIT { contents } => contents.to_string(),
            _ => unimplemented!(),
        }
    }
    /// 識別子トークンだと仮定して，名前を受け取る
    pub fn copy_name(&self) -> String {
        match &self.k {
            TokenKind::IDENTIFIER { name } => name.to_string(),
            _ => unimplemented!(),
        }
    }

    pub fn get_kind(&self) -> &TokenKind {
        &self.k
    }

    /// Tokenizerが無視するトークンの種類
    pub fn should_ignore(&self) -> bool {
        match &self.k {
            TokenKind::BLANK | TokenKind::NEWLINE | TokenKind::COMMENT { contents: _ } => true,
            _ => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> FR {
        write!(f, "{}: {}", self.p, self.k)
    }
}
