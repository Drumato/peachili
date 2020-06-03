use std::collections::BTreeMap;

use crate::common::{error, option, position};
use crate::compiler::general::resource as res;

#[allow(dead_code)]
pub struct Parser<'a> {
    build_option: &'a option::BuildOption,
    cur_token: usize,
    next_token: usize,

    row: usize,
    col: usize,

    errors: Vec<error::CompileError>,
    tokens: Vec<res::Token>,
    root: res::ASTRoot,
}

impl<'a> Parser<'a> {
    pub fn add_pfunction(&mut self, name_id: res::PStringId, func: res::PFunction) {
        self.root.add_pfunction(name_id, func);
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
            errors: Vec::new(),
            root: Default::default(),
        }
    }

    pub fn give_root_and_errors(self) -> (res::ASTRoot, Vec<error::CompileError>) {
        (self.root, self.errors)
    }

    pub fn detect_error(&mut self, e: error::CompileError) {
        self.errors.push(e);

        loop {
            if self.eat_if_matched(&res::TokenKind::SEMICOLON) {
                break;
            }
            self.progress();
        }
    }

    pub fn get_typedefs(&self) -> &BTreeMap<res::PStringId, res::PType> {
        self.root.get_typedefs()
    }

    pub fn add_typedef(&mut self, type_name_id: res::PStringId, src_type: res::PType) {
        self.root.add_typedef(type_name_id, src_type);
    }
    pub fn add_local_var_to(
        &mut self,
        func_name_id: res::PStringId,
        var_name_ids: Vec<res::PStringId>,
        pvar: res::PVariable,
    ) {
        self.root.add_local_var_to(func_name_id, var_name_ids, pvar);
    }
    pub fn add_string_to(
        &mut self,
        func_name_id: res::PStringId,
        contents_id: res::PStringId,
        hash: u64,
    ) {
        self.root.add_string_to(func_name_id, contents_id, hash);
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
    pub fn replace_statements(&mut self, name_id: res::PStringId, stmts: Vec<res::StatementNode>) {
        self.root.replace_statement(name_id, stmts);
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
