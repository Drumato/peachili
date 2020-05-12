use std::collections::BTreeMap;

use crate::common::{option, position};
use crate::compiler::resource as res;

#[allow(dead_code)]
pub struct Parser<'a> {
    build_option: &'a option::BuildOption,
    cur_token: usize,
    next_token: usize,

    row: usize,
    col: usize,

    tokens: Vec<res::Token>,
    functions: BTreeMap<String, res::PFunction>,
}

impl<'a> Parser<'a> {
    pub fn add_pfunction(&mut self, name: String, func: res::PFunction) {
        self.functions.insert(name, func);
    }
}

impl<'a> Parser<'a> {
    pub fn new(opt: &'a option::BuildOption, tks: Vec<res::Token>) -> Self {
        Self {
            build_option: opt,
            tokens: tks,
            cur_token: 0,
            next_token: 1,
            row: 1,
            col: 1,
            functions: BTreeMap::new(),
        }
    }
    pub fn give_functions(self) -> BTreeMap<String, res::PFunction> {
        self.functions
    }

    pub fn add_local_var_to(&mut self, func_name: &str, var_name: String, pvar: res::PVariable) {
        if let Some(p_func) = self.functions.get_mut(func_name) {
            p_func.add_local(var_name, pvar);
        }
    }
    pub fn add_string_to(&mut self, func_name: &str, contents: String, hash: u64) {
        if let Some(p_func) = self.functions.get_mut(func_name) {
            p_func.add_string(contents, hash);
        }
    }

    pub fn cur_token_is(&self, tk: &res::TokenKind) -> bool {
        self.current_token_kind() == tk
    }

    pub fn current_token_kind(&self) -> &res::TokenKind {
        &self.tokens[self.cur_token].kind
    }
    pub fn get_specified_token(&self, offset: usize) -> &res::TokenKind {
        &self.tokens[offset].kind
    }
    pub fn replace_statements(&mut self, name: &str, stmts: Vec<res::StatementNode>) {
        if let Some(p_func) = self.functions.get_mut(name) {
            p_func.replace_statements(stmts);
        }
    }

    pub fn save_current_offset(&self) -> usize {
        self.cur_token
    }
    pub fn current_token(&self) -> &res::Token {
        &self.tokens[self.cur_token]
    }

    pub fn eat_if_matched(&mut self, tk: &res::TokenKind) -> bool {
        if self.current_token_kind() != tk {
            return false;
        }

        self.progress();
        true
    }

    pub fn expect(&mut self, tk: res::TokenKind) {
        let cur_pos = self.current_position();
        let tk_str = tk.to_str();

        if !self.eat_if_matched(&tk) {
            panic!(
                "{} expected {} but got {}",
                cur_pos,
                tk_str,
                self.current_token_kind().to_str()
            );
        }
    }
    pub fn progress(&mut self) {
        self.cur_token += 1;
        self.next_token += 1;

        let cur_tok_pos = self.current_position();
        let (row, col) = cur_tok_pos.get_pos();
        self.row = row;
        self.col = col;
    }

    pub fn current_position(&self) -> position::Position {
        let (r, c) = self.current_token().get_pos().get_pos();
        position::Position::new(r, c)
    }
}
