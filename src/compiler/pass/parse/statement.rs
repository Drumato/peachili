use crate::compiler::resource as res;

impl<'a> res::Parser<'a> {
    pub fn compound_statement(&mut self) -> Vec<res::StatementNode> {
        if !self.eat_if_matched(&res::TokenKind::LBRACE) {
            let cur_pos = self.current_position();
            panic!(
                "{} compound-statement must be started with '{{', got '{}'",
                cur_pos,
                self.current_token_kind().to_str()
            );
        }

        let mut statements: Vec<res::StatementNode> = Vec::new();

        loop {
            if self.eat_if_matched(&res::TokenKind::RBRACE) {
                break;
            }

            statements.push(self.statement());
        }

        statements
    }

    pub fn statement(&mut self) -> res::StatementNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::RETURN => self.return_statement(),
            res::TokenKind::IFRET => self.ifret_statement(),
            res::TokenKind::DECLARE => self.vardecl_statement(),
            res::TokenKind::COUNTUP => self.countup_statement(),
            _ => self.expression_statement(),
        }
    }

    pub fn return_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression();

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_return(expr, cur_pos)
    }

    pub fn ifret_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression();
        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_ifret(expr, cur_pos)
    }

    pub fn vardecl_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let name = self.expect_name();
        let ptype = self.expect_ptype();

        let declared_var = res::PVariable::new_local(ptype);
        self.add_local_var_to_curfunc(name, declared_var);

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_vardecl(cur_pos)
    }

    pub fn countup_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let ident = self.primary();
        let ptype = self.expect_ptype();

        let loop_var = res::PVariable::new_local(ptype);
        self.add_local_var_to_curfunc(ident.copy_ident_name(), loop_var);

        self.expect(res::TokenKind::FROM);
        let start_expr = self.expression();

        self.expect(res::TokenKind::TO);
        let end_expr = self.expression();

        let body = self.compound_statement();

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_countup(ident, start_expr, end_expr, body, cur_pos)
    }

    pub fn expression_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        let expr = self.expression();
        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_expr(expr, cur_pos)
    }
}
