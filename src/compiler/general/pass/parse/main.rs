use std::collections::BTreeMap;

use crate::common::{error, module, operate, option, position};
use crate::compiler::general::resource as res;

type CompileErrors = Vec<error::CompileError>;

pub fn parse_phase(
    build_option: &option::BuildOption,
    module_path: &str,
    module_id: module::ModuleId,
    tokens: Vec<res::Token>,
) -> res::ASTRoot {
    let (root, parse_errors) = parse(build_option, module_path, module_id, tokens);
    if !parse_errors.is_empty() {
        operate::emit_all_errors_and_exit(&parse_errors, module_path, build_option);
    }

    root
}

fn parse(
    opt: &option::BuildOption,
    module_path: &str,
    module_id: module::ModuleId,
    tokens: Vec<res::Token>,
) -> (res::ASTRoot, CompileErrors) {
    let mut parser: res::Parser = res::Parser::new(opt, tokens);
    parser.toplevel(module_path, module_id);

    parser.give_root_and_errors()
}

impl<'a> res::Parser<'a> {
    fn toplevel(&mut self, module_path: &str, module_id: module::ModuleId) {
        self.skip_require();

        // 関数列のパース
        loop {
            let cur_token = self.current_token();
            match &cur_token.kind {
                res::TokenKind::FUNC => self.function(module_path.to_string(), module_id),
                res::TokenKind::PUBTYPE => self.type_alias(),
                _ => break,
            }
        }
    }

    fn type_alias(&mut self) {
        let def_func_pos = self.current_position();
        self.progress();

        let type_name_id = self.expect_name();
        if type_name_id.is_none() {
            return;
        }

        self.expect(res::TokenKind::ASSIGN);

        let src_type = self.expect_ptype();
        self.expect_semicolon(def_func_pos);

        self.add_typedef(type_name_id.unwrap(), src_type);
    }

    fn function(&mut self, module_path: String, module_id: module::ModuleId) {
        let def_func_pos = self.current_position();
        self.progress();

        let name_id = self.expect_name();
        if name_id.is_none() {
            return;
        }

        // 引数
        let mut args: Vec<res::PStringId> = Vec::new();
        let mut arg_map: BTreeMap<Vec<res::PStringId>, res::PVariable> = BTreeMap::new();

        self.expect(res::TokenKind::LPAREN);

        loop {
            if self.eat_if_matched(&res::TokenKind::RPAREN) {
                break;
            }
            let arg_name_id = self.expect_name();
            if arg_name_id.is_none() {
                return;
            }

            let arg_type = self.expect_ptype();

            let arg_var = res::PVariable::new_local(arg_type, false);

            assert!(arg_map
                .insert(vec![arg_name_id.unwrap()], arg_var)
                .is_none());

            self.eat_if_matched(&res::TokenKind::COMMA);
            args.push(arg_name_id.unwrap());
        }

        let return_type = self.expect_ptype();

        let mut defined_func = res::PFunction::new(
            name_id.unwrap(),
            return_type,
            args,
            def_func_pos,
            module_path,
            module_id,
        );
        defined_func.set_locals(arg_map);

        self.add_pfunction(name_id.unwrap(), defined_func);

        let statements = self.compound_statement(name_id.unwrap());

        self.replace_statements(name_id.unwrap(), statements);
    }

    pub fn expect_ptype(&mut self) -> res::PType {
        let cur_pos = self.current_position();
        let ptype_kind = self.current_token_kind().clone();

        if let res::TokenKind::IDENTIFIER(_name) = ptype_kind {
            let ident = self.expect_identifier();

            return res::PType::new_unresolved(ident);
        }

        self.progress();

        match ptype_kind {
            res::TokenKind::INT64 => res::PType::new_int64(),
            res::TokenKind::UINT64 => res::PType::new_uint64(),
            res::TokenKind::NORETURN => res::PType::new_noreturn(),
            res::TokenKind::STR => res::PType::new_str(),
            res::TokenKind::BOOLEAN => res::PType::new_boolean(),
            res::TokenKind::ASTERISK => {
                let inner_type = self.expect_ptype();
                res::PType::new_pointer(inner_type)
            }
            _ => {
                self.detect_error(error::CompileError::got_invalid_ptype(cur_pos));
                res::PType::new_invalid()
            }
        }
    }

    pub fn expect_name(&mut self) -> Option<res::PStringId> {
        let cur_pos = self.current_position();

        let cur_ident = self.current_token().get_ident_id();
        self.progress();

        if cur_ident.is_none() {
            self.detect_error(error::CompileError::statement_must_be_ended_with_semicolon(
                cur_pos,
            ));
            return None;
        }

        Some(cur_ident.unwrap())
    }

    pub fn expect_semicolon(&mut self, stmt_pos: position::Position) {
        if !self.eat_if_matched(&res::TokenKind::SEMICOLON) {
            self.detect_error(error::CompileError::statement_must_be_ended_with_semicolon(
                stmt_pos,
            ));
        }
    }

    pub fn expect_identifier(&mut self) -> res::IdentName {
        let cur_pos = self.current_position();
        let cur_ident = self.current_token().get_ident_id();
        if cur_ident.is_none() {
            self.detect_error(error::CompileError::expected_identifier(cur_pos));
            panic!();
        }

        self.progress();

        let mut base = res::IdentName::new_terminated(cur_ident.unwrap());

        if !self.cur_token_is(&res::TokenKind::DOUBLECOLON) {
            return base;
        }

        self.progress();

        let next = self.expect_identifier();
        base.append_next(next);

        base
    }

    pub fn skip_require(&mut self) {
        // require ( "module-name" * ) を取り除く
        if self.cur_token_is(&res::TokenKind::REQUIRE) {
            loop {
                if self.eat_if_matched(&res::TokenKind::RPAREN) {
                    break;
                }
                self.progress();
            }
        }
    }
}
