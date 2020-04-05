use crate::common::{module, option, position};
use crate::compiler::resource as res;

#[allow(dead_code)]
pub struct Parser<'a> {
    build_option: &'a option::BuildOption,
    cur_token: usize,
    next_token: usize,

    row: usize,
    col: usize,

    tokens: Vec<res::Token>,
    functions: Vec<res::PFunction>,
    this_mod: &'a mut module::Module,
}

impl<'a> Parser<'a> {
    pub fn add_pfunction(&mut self, func: res::PFunction) {
        self.functions.push(func);
    }
}

impl<'a> Parser<'a> {
    pub fn new(
        opt: &'a option::BuildOption,
        tks: Vec<res::Token>,
        cur_mod: &'a mut module::Module,
    ) -> Self {
        Self {
            build_option: opt,
            tokens: tks,
            cur_token: 0,
            next_token: 1,
            row: 1,
            col: 1,
            functions: Vec::new(),
            this_mod: cur_mod,
        }
    }
    pub fn give_functions(self) -> Vec<res::PFunction> {
        self.functions
    }

    pub fn cur_token_is(&self, tk: res::TokenKind) -> bool {
        self.current_token_kind() == tk
    }

    pub fn current_token_kind(&self) -> res::TokenKind {
        self.tokens[self.cur_token].kind.clone()
    }

    pub fn current_token(&self) -> &res::Token {
        &self.tokens[self.cur_token]
    }

    pub fn eat_if_matched(&mut self, tk: res::TokenKind) -> bool {
        if self.current_token_kind() != tk {
            return false;
        }

        self.progress();
        return true;
    }

    pub fn expect(&mut self, tk: res::TokenKind) {
        let cur_pos = self.current_position();
        let tk_str = tk.to_str();

        if !self.eat_if_matched(tk) {
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
