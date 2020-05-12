use crate::compiler::resource as res;

impl<'a> res::Parser<'a> {
    pub fn compound_statement(&mut self, func_name: &str) -> Vec<res::StatementNode> {
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

            statements.push(self.statement(func_name));
        }

        statements
    }

    pub fn statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::RETURN => self.return_statement(func_name),
            res::TokenKind::IFRET => self.ifret_statement(func_name),
            res::TokenKind::DECLARE => self.vardecl_statement(func_name),
            res::TokenKind::COUNTUP => self.countup_statement(func_name),
            res::TokenKind::ASM => self.asm_statement(),
            _ => self.expression_statement(func_name),
        }
    }

    fn return_statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression(func_name);

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_return(expr, cur_pos)
    }

    fn ifret_statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression(func_name);
        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_ifret(expr, cur_pos)
    }

    fn vardecl_statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let name = self.expect_name();
        let ptype = self.expect_ptype();

        let declared_var = res::PVariable::new_local(ptype);
        self.add_local_var_to(func_name, name, declared_var);

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_vardecl(cur_pos)
    }

    fn countup_statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let ident = self.primary(func_name);
        let ptype = self.expect_ptype();

        let loop_var = res::PVariable::new_local(ptype);
        self.add_local_var_to(func_name, ident.copy_ident_name(), loop_var);

        self.expect(res::TokenKind::FROM);
        let start_expr = self.expression(func_name);

        self.expect(res::TokenKind::TO);
        let end_expr = self.expression(func_name);

        let body = self.compound_statement(func_name);

        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_countup(ident, start_expr, end_expr, body, cur_pos)
    }

    fn expression_statement(&mut self, func_name: &str) -> res::StatementNode {
        let cur_pos = self.current_position();
        let expr = self.expression(func_name);
        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_expr(expr, cur_pos)
    }

    fn asm_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        if !self.eat_if_matched(&res::TokenKind::LBRACE) {
            panic!("expected {{");
        }

        let mut asms: Vec<String> = Vec::new();

        loop {
            if self.eat_if_matched(&res::TokenKind::RBRACE) {
                break;
            }

            let (contents, hash, cur_pos) = self.parse_string_literal();
            let asm_contents_node = res::ExpressionNode::new_strlit(contents, hash, cur_pos);

            asms.push(asm_contents_node.copy_str_contents());
            self.eat_if_matched(&res::TokenKind::COMMA);
        }
        self.expect_semicolon(&cur_pos);

        res::StatementNode::new_asm(asms, cur_pos)
    }
}
