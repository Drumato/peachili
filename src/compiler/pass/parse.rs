use crate::common::{module, option};
use crate::compiler::resource as res;

pub fn parse(
    opt: &option::BuildOption,
    tokens: Vec<res::Token>,
    mut this_mod: module::Module,
) -> (module::Module, Vec<res::PFunction>) {
    let functions: Vec<res::PFunction>;
    {
        let mut parser: res::Parser = res::Parser::new(opt, tokens, &mut this_mod);
        parser.toplevel();

        functions = parser.give_functions()
    }
    (this_mod, functions)
}

impl<'a> res::Parser<'a> {}

impl<'a> res::Parser<'a> {
    fn toplevel(&mut self) {
        self.skip_require();

        // 関数列のパース
        loop {
            if !self.cur_token_is(res::TokenKind::FUNC) {
                break;
            }

            self.function();
        }
    }

    fn function(&mut self) {
        let def_func_pos = self.current_token().get_pos();
        self.progress();

        let name = self.expect_identifier();

        // 引数は後で
        self.expect(res::TokenKind::LPAREN);
        self.expect(res::TokenKind::RPAREN);

        let return_type = self.expect_ptype();

        let statements = self.compound_statement();

        let defined_func = res::PFunction::new(name, return_type, def_func_pos, statements);
        self.add_pfunction(defined_func);
    }

    fn compound_statement(&mut self) -> Vec<res::StatementNode> {
        if !self.eat_if_matched(res::TokenKind::LBRACE) {
            let cur_pos = self.current_position();
            panic!(
                "{} compound-statement must be started with '{{', got '{}'",
                cur_pos,
                self.current_token_kind().to_str()
            );
        }

        let mut statements: Vec<res::StatementNode> = Vec::new();

        loop {
            if self.eat_if_matched(res::TokenKind::RBRACE) {
                break;
            }

            statements.push(self.statement());
        }

        statements
    }

    fn statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::RETURN => self.return_statement(),
            _ => panic!(
                "{} unexpected {} when statement started",
                cur_pos,
                cur_kind.to_str()
            ),
        }
    }

    fn return_statement(&mut self) -> res::StatementNode {
        let cur_pos = self.current_position();
        self.progress();

        let expr = self.expression();

        if !self.eat_if_matched(res::TokenKind::SEMICOLON) {
            panic!("{} return statement must be end with ';'", cur_pos);
        }

        res::StatementNode::new_return(expr, cur_pos)
    }

    fn expression(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        // 適当に整数リテラルやっとく
        let cur_int_value = self.current_token().get_int_value();
        self.progress();

        if cur_int_value.is_none() {
            panic!("{} expected integer-literal", cur_pos);
        }

        res::ExpressionNode::new_intlit(cur_int_value.unwrap(), cur_pos)
    }

    fn expect_ptype(&mut self) -> res::PType {
        let cur_kind = self.current_token_kind();
        self.progress();

        match cur_kind {
            res::TokenKind::INT64 => res::PType::new_int64(),
            _ => panic!("can't find such a type -> {}", cur_kind.to_str()),
        }
    }

    // TODO: IdentNameを返り値に
    fn expect_identifier(&mut self) -> String {
        let cur_pos = self.current_position();

        let cur_ident = self.current_token().copy_ident_name();
        self.progress();

        if cur_ident.is_none() {
            panic!("{} expected identifier", cur_pos);
        }

        cur_ident.unwrap()
    }

    fn skip_require(&mut self) {
        // require ( "module-name" * ) を取り除く
        if self.cur_token_is(res::TokenKind::REQUIRE) {
            loop {
                if self.eat_if_matched(res::TokenKind::RPAREN) {
                    break;
                }
                self.progress();
            }
        }
    }
}
