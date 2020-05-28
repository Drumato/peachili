use std::collections::BTreeMap;
use std::time;

use colored::*;

use crate::common::{module, option, position};
use crate::compiler::general::resource as res;

pub fn parse_phase(
    build_option: &option::BuildOption,
    module_path: &str,
    module_id: module::ModuleId,
    tokens: Vec<res::Token>,
) -> res::ASTRoot {
    let start = time::Instant::now();

    // TODO: パースエラー
    let root = parse(build_option, module_path, module_id, tokens);

    let end = time::Instant::now();

    if build_option.verbose {
        eprintln!(
            "    {}: parse {} done in {:?}",
            "STEP2".bold().green(),
            module_path,
            end - start
        );
    }
    root
}

// TODO: 文字列リテラル群を返すように調整する．
// パーサに文字列リテラルを格納するメンバを作って，Vec<PFunction>と一緒に返す
fn parse(
    opt: &option::BuildOption,
    module_path: &str,
    module_id: module::ModuleId,
    tokens: Vec<res::Token>,
) -> res::ASTRoot {
    let mut parser: res::Parser = res::Parser::new(opt, tokens);
    parser.toplevel(module_path, module_id);

    parser.give_root()
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

        self.expect(res::TokenKind::ASSIGN);

        let src_type = self.expect_ptype();
        self.expect_semicolon(&def_func_pos);

        self.add_typedef(type_name_id, src_type);
    }

    fn function(&mut self, module_path: String, module_id: module::ModuleId) {
        let def_func_pos = self.current_position();
        self.progress();

        let name_id = self.expect_name();

        // 引数
        let mut args: Vec<res::PStringId> = Vec::new();
        let mut arg_map: BTreeMap<Vec<res::PStringId>, res::PVariable> = BTreeMap::new();

        self.expect(res::TokenKind::LPAREN);

        loop {
            if self.eat_if_matched(&res::TokenKind::RPAREN) {
                break;
            }
            let arg_name_id = self.expect_name();

            let arg_type = self.expect_ptype();

            let arg_var = res::PVariable::new_local(arg_type, false);

            assert!(arg_map.insert(vec![arg_name_id], arg_var).is_none());

            self.eat_if_matched(&res::TokenKind::COMMA);
            args.push(arg_name_id);
        }

        let return_type = self.expect_ptype();

        let mut defined_func = res::PFunction::new(
            name_id,
            return_type,
            args,
            def_func_pos,
            module_path,
            module_id,
        );
        defined_func.set_locals(arg_map);

        self.add_pfunction(name_id, defined_func);

        let statements = self.compound_statement(name_id);

        self.replace_statements(name_id, statements);
    }

    pub fn expect_ptype(&mut self) -> res::PType {
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
            _ => panic!("got invalid ptype -> {:?}", ptype_kind),
        }
    }

    pub fn expect_name(&mut self) -> res::PStringId {
        let cur_pos = self.current_position();

        let cur_ident = self.current_token().get_ident_id();
        self.progress();

        if cur_ident.is_none() {
            panic!("{} name must be an identifier.", cur_pos);
        }

        cur_ident.unwrap()
    }

    pub fn expect_semicolon(&mut self, stmt_pos: &position::Position) {
        if !self.eat_if_matched(&res::TokenKind::SEMICOLON) {
            panic!("{} statement must be end with ';'", stmt_pos);
        }
    }

    pub fn expect_identifier(&mut self) -> res::IdentName {
        let cur_pos = self.current_position();
        let cur_ident = self.current_token().get_ident_id();
        if cur_ident.is_none() {
            panic!("{} expected identifier.", cur_pos);
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
