use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::compiler::resource as res;

impl<'a> res::Parser<'a> {
    pub fn expression(&mut self) -> res::ExpressionNode {
        if self.cur_token_is(&res::TokenKind::IF) {
            return self.if_expression();
        }
        self.assignment()
    }

    pub fn if_expression(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        self.progress();

        self.expect(res::TokenKind::LPAREN);

        let condition = self.expression();
        self.expect(res::TokenKind::RPAREN);

        let body = self.compound_statement();

        if !self.cur_token_is(&res::TokenKind::ELSE) {
            return res::ExpressionNode::new_if(condition, body, cur_pos);
        }

        self.progress();

        let else_body = self.compound_statement();

        res::ExpressionNode::new_ifelse(condition, body, else_body, cur_pos)
    }

    pub fn assignment(&mut self) -> res::ExpressionNode {
        let lval = self.additive();

        if !self.cur_token_is(&res::TokenKind::ASSIGN) {
            return lval;
        }

        let cur_pos = self.current_position();
        self.progress();

        let rval = self.expression();

        res::ExpressionNode::new_assign(lval, rval, cur_pos)
    }

    pub fn additive(&mut self) -> res::ExpressionNode {
        let mut lop = self.multiplicative();

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::PLUS) {
                lop = res::ExpressionNode::new_add(lop, self.multiplicative(), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::MINUS) {
                lop = res::ExpressionNode::new_sub(lop, self.multiplicative(), cur_pos);
            } else {
                break;
            }
        }
        lop
    }

    pub fn multiplicative(&mut self) -> res::ExpressionNode {
        let mut lop = self.unary();

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::ASTERISK) {
                lop = res::ExpressionNode::new_mul(lop, self.unary(), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::SLASH) {
                lop = res::ExpressionNode::new_div(lop, self.unary(), cur_pos);
            } else {
                break;
            }
        }

        lop
    }

    pub fn unary(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();

        if self.eat_if_matched(&res::TokenKind::MINUS) {
            return res::ExpressionNode::new_neg(self.primary(), cur_pos);
        }
        self.eat_if_matched(&res::TokenKind::PLUS);

        self.primary()
    }

    pub fn primary(&mut self) -> res::ExpressionNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::INTEGER(_v) => self.integer_literal(),
            res::TokenKind::STRLIT(_contents) => self.string_literal(),
            res::TokenKind::IDENTIFIER(_name) => self.identifier(),
            _ => panic!("unexpected {} in primary", cur_kind.to_str()),
        }
    }

    pub fn integer_literal(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_int_value = self.current_token().get_int_value();
        self.progress();

        // primary() で整数リテラルであることを検査しているのでunwrap()してよい．
        res::ExpressionNode::new_intlit(cur_int_value.unwrap(), cur_pos)
    }

    pub fn string_literal(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_str_contents = self.current_token().copy_strlit_contents();
        self.progress();

        let mut hasher = DefaultHasher::new();
        let contents = cur_str_contents.unwrap();
        contents.hash(&mut hasher);

        if !self.asm_mode() {
            self.add_string_to_curfunc(contents.clone(), hasher.finish());
        }

        // primary() で文字列リテラルであることを検査しているのでunwrap()してよい．
        res::ExpressionNode::new_strlit(contents, hasher.finish(), cur_pos)
    }

    pub fn identifier(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_ident = self.expect_identifier();

        if !self.eat_if_matched(&res::TokenKind::LPAREN) {
            return res::ExpressionNode::new_ident(cur_ident, cur_pos);
        }

        let mut args: Vec<res::ExpressionNode> = Vec::new();

        loop {
            if self.eat_if_matched(&res::TokenKind::RPAREN) {
                break;
            }

            args.push(self.expression());

            self.eat_if_matched(&res::TokenKind::COMMA);
        }

        res::ExpressionNode::new_call(cur_ident, args, cur_pos)
    }
}
