use crate::common::{operate, option, position};
use crate::compiler::resource as res;

#[allow(dead_code)]
pub struct Lexer<'a> {
    build_option: &'a option::BuildOption,
    col: usize,
    row: usize,
    contents: String,
    tokens: Vec<res::Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(opt: &'a option::BuildOption, program: String) -> Self {
        Self {
            build_option: opt,
            col: 1,
            row: 1,
            contents: program,
            tokens: Vec::new(),
        }
    }
    pub fn give_token(self) -> Vec<res::Token> {
        self.tokens
    }

    pub fn add_token(&mut self, t: res::Token) {
        self.tokens.push(t);
    }

    pub fn offset_overruns_contents_length(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn cur_position(&self) -> position::Position {
        position::Position::new(self.row, self.col)
    }
    pub fn set_position(&mut self, pos: position::Position) {
        let (row, col) = pos.get_pos();
        self.row = row;
        self.col = col;
    }

    pub fn skip_offset(&mut self, len: usize) {
        self.col += len;
        self.consume_contents(len);
    }

    pub fn consume_contents(&mut self, len: usize) {
        self.contents.drain(..len);
    }
    pub fn skip_crlf(&mut self) {
        self.col = 1;
        self.row += 1;
        self.contents.drain(..1);
    }

    pub fn cut_contents(&self, f: fn(&char) -> bool) -> String {
        operate::take_conditional_string(&self.contents, f)
    }

    pub fn contents_starts_with(&self, s: &str) -> bool {
        self.contents.starts_with(s)
    }
    pub fn last_token(&self) -> &res::Token {
        &self.tokens[self.tokens.len() - 1]
    }

    pub fn cur_offset_char(&self) -> char {
        self.contents.as_bytes()[0] as char
    }
}
