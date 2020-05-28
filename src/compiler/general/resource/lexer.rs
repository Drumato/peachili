use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::common::{error, operate, option, position};
use crate::compiler::general::resource as res;

#[allow(dead_code)]
pub struct Lexer<'a> {
    build_option: &'a option::BuildOption,
    col: usize,
    row: usize,
    contents: String,
    tokens: Vec<res::Token>,
    errors: Vec<error::CompileError>,
    const_pool: Arc<Mutex<res::ConstAllocator>>,
    buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(
        opt: &'a option::BuildOption,
        program: String,
        pool: Arc<Mutex<res::ConstAllocator>>,
        buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
    ) -> Self {
        Self {
            build_option: opt,
            col: 1,
            row: 1,
            contents: program,
            tokens: Vec::new(),
            errors: Vec::new(),
            const_pool: pool,
            buffer_cache,
        }
    }

    pub fn check_already_alloced(&self, st: &str) -> bool {
        self.buffer_cache.lock().unwrap().contains_key(st)
    }
    pub fn get_cached_id(&self, st: &str) -> res::PStringId {
        *self.buffer_cache.lock().unwrap().get(st).unwrap()
    }
    pub fn insert_new_buffer(&mut self, st: String) -> res::PStringId {
        let id = self.alloc_string(st.clone());
        self.buffer_cache.lock().unwrap().insert(st, id);
        id
    }
    pub fn give_token_and_const_arena(self) -> (Vec<res::Token>, Arc<Mutex<res::ConstAllocator>>) {
        (self.tokens, self.const_pool)
    }
    fn alloc_string(&mut self, v: String) -> res::PStringId {
        self.const_pool.lock().unwrap().alloc_string(v)
    }
    pub fn copy_errors(&self) -> Vec<error::CompileError> {
        self.errors.clone()
    }

    pub fn detect_error(&mut self, e: error::CompileError) {
        self.errors.push(e);
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
        if self.contents.len() < len {
            return;
        }
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
