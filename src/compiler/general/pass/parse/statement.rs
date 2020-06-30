use crate::compiler::general::resource as res;

impl<'a> res::Parser<'a> {
    pub fn compound_statement(&mut self, func_name_id: res::PStringId) -> Vec<res::StatementNode> {
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

            statements.push(self.statement(func_name_id));
        }

        statements
    }

    pub fn statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::RETURN => self.return_statement(func_name_id),
            res::TokenKind::IFRET => self.ifret_statement(func_name_id),
            res::TokenKind::DECLARE => self.vardecl_statement(func_name_id),
            res::TokenKind::COUNTUP => self.countup_statement(func_name_id),
            res::TokenKind::ASM => self.asm_statement(),
            res::TokenKind::VARINIT => self.varinit_statement(func_name_id),
            res::TokenKind::CONST => self.const_statement(func_name_id),
            _ => self.expression_statement(func_name_id),
        }
    }

    fn return_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression(func_name_id);

        self.expect_semicolon(cur_pos.clone());

        res::StatementNode::new_return(expr, cur_pos)
    }

    fn ifret_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression(func_name_id);
        self.expect_semicolon(cur_pos.clone());

        res::StatementNode::new_ifret(expr, cur_pos)
    }

    fn vardecl_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let (vardecl_pos, var_name_id) = self.skip_keyword_and_ident();

        let ptype = self.expect_ptype();

        let declared_var = res::PVariable::new_local(ptype, false);
        self.add_local_var_to(func_name_id, vec![var_name_id], declared_var);

        self.expect_semicolon(vardecl_pos.clone());

        res::StatementNode::new_vardecl(vardecl_pos)
    }

    fn countup_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let ident = self.primary(func_name_id);
        let ptype = self.expect_ptype();

        let loop_var = res::PVariable::new_local(ptype, false);
        self.add_local_var_to(func_name_id, ident.get_ident_ids(), loop_var);

        self.expect(res::TokenKind::FROM);
        let start_expr = self.expression(func_name_id);

        self.expect(res::TokenKind::TO);
        let end_expr = self.expression(func_name_id);

        let body = self.compound_statement(func_name_id);

        self.expect_semicolon(cur_pos.clone());

        res::StatementNode::new_countup(ident, start_expr, end_expr, body, cur_pos)
    }

    fn expression_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let cur_pos = self.current_position();
        let expr = self.expression(func_name_id);
        self.expect_semicolon(cur_pos.clone());

        res::StatementNode::new_expr(expr, cur_pos)
    }

    fn asm_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        if !self.eat_if_matched(&res::TokenKind::LBRACE) {
            panic!("expected {{");
        }

        let mut asms: Vec<res::PStringId> = Vec::new();

        loop {
            if self.eat_if_matched(&res::TokenKind::RBRACE) {
                break;
            }

            let (contents, hash, cur_pos) = self.parse_string_literal();
            let asm_contents_node = res::ExpressionNode::new_strlit(contents, hash, cur_pos);

            asms.push(asm_contents_node.get_str_id());
            self.eat_if_matched(&res::TokenKind::COMMA);
        }
        self.expect_semicolon(cur_pos.clone());

        res::StatementNode::new_asm(asms, cur_pos)
    }

    fn varinit_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let st_pos = self.current_position();
        self.progress();

        let ident = self.primary(func_name_id);
        let ptype = self.expect_ptype();

        let declared_var = res::PVariable::new_local(ptype, false);
        self.add_local_var_to(func_name_id, ident.get_ident_ids(), declared_var);

        self.expect(res::TokenKind::ASSIGN);

        let init_expression = self.expression(func_name_id);

        self.expect_semicolon(st_pos.clone());

        res::StatementNode::new_varinit(ident, init_expression, st_pos)
    }

    fn const_statement(&mut self, func_name_id: res::PStringId) -> res::StatementNode {
        let st_pos = self.current_position();
        self.progress();

        let ident = self.primary(func_name_id);
        let ptype = self.expect_ptype();

        let declared_var = res::PVariable::new_local(ptype, true);
        self.add_local_var_to(func_name_id, ident.get_ident_ids(), declared_var);

        self.expect(res::TokenKind::ASSIGN);

        let init_expression = self.expression(func_name_id);

        self.expect_semicolon(st_pos.clone());

        res::StatementNode::new_varinit(ident, init_expression, st_pos)
    }
}
