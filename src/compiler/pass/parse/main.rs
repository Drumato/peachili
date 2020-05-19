use std::collections::BTreeMap;
use std::time;

use colored::*;

use crate::common::{option, position};
use crate::compiler::resource as res;

pub fn parse_phase(
    build_option: &option::BuildOption,
    module_path: &str,
    tokens: Vec<res::Token>,
) -> res::ASTRoot {
    let start = time::Instant::now();

    // TODO: パースエラー
    let root = parse(build_option, module_path, tokens);

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
fn parse(opt: &option::BuildOption, module_path: &str, tokens: Vec<res::Token>) -> res::ASTRoot {
    let mut parser: res::Parser = res::Parser::new(opt, tokens);
    parser.toplevel(module_path);

    parser.give_root()
}

impl<'a> res::Parser<'a> {
    fn toplevel(&mut self, module_path: &str) {
        self.skip_require();

        // 関数列のパース
        loop {
            let cur_token = self.current_token();
            match &cur_token.kind {
                res::TokenKind::FUNC => self.function(module_path.to_string()),
                res::TokenKind::PUBTYPE => self.type_alias(),
                _ => break,
            }
        }
    }

    fn type_alias(&mut self) {
        let def_func_pos = self.current_position();
        self.progress();

        let type_name = self.expect_name();

        self.expect(res::TokenKind::ASSIGN);

        let src_type = self.expect_ptype();
        self.expect_semicolon(&def_func_pos);

        self.add_typedef(type_name, src_type);
    }

    fn function(&mut self, module_path: String) {
        let def_func_pos = self.current_position();
        self.progress();

        let name = self.expect_name();

        // 引数
        let mut args: Vec<String> = Vec::new();
        let mut arg_map: BTreeMap<String, res::PVariable> = BTreeMap::new();

        self.expect(res::TokenKind::LPAREN);
        loop {
            if self.eat_if_matched(&res::TokenKind::RPAREN) {
                break;
            }
            let arg_name = res::IdentName::correct_name(&self.expect_identifier());
            let arg_type = self.expect_ptype();

            let arg_var = res::PVariable::new_local(arg_type);

            arg_map.insert(arg_name.clone(), arg_var);

            self.eat_if_matched(&res::TokenKind::COMMA);
            args.push(arg_name);
        }

        let return_type = self.expect_ptype();

        let mut defined_func =
            res::PFunction::new(name.clone(), return_type, args, def_func_pos, module_path);
        defined_func.set_locals(arg_map);
        self.add_pfunction(name.clone(), defined_func);

        let statements = self.compound_statement(&name);

        self.replace_statements(&name, statements);
    }

    pub fn expect_ptype(&mut self) -> res::PType {
        let ptype_offset = self.save_current_offset();
        self.progress();

        let ptype_kind = self.get_specified_token(ptype_offset);
        match ptype_kind {
            res::TokenKind::INT64 => res::PType::new_int64(),
            res::TokenKind::UINT64 => res::PType::new_uint64(),
            res::TokenKind::NORETURN => res::PType::new_noreturn(),
            res::TokenKind::STR => res::PType::new_str(),
            res::TokenKind::BOOLEAN => res::PType::new_boolean(),
            res::TokenKind::IDENTIFIER(name) => res::PType::new_unresolved(name.to_string()),
            _ => res::PType::new_unresolved(String::new()),
        }
    }

    pub fn expect_name(&mut self) -> String {
        let cur_pos = self.current_position();

        let cur_ident = self.current_token().copy_ident_name();
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
        let cur_ident = self.current_token().copy_ident_name();
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
