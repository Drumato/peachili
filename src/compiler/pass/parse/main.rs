use std::collections::BTreeMap;

use crate::common::{option, position};
use crate::compiler::resource as res;

// TODO: 文字列リテラル群を返すように調整する．
// パーサに文字列リテラルを格納するメンバを作って，Vec<PFunction>と一緒に返す
pub fn parse(opt: &option::BuildOption, tokens: Vec<res::Token>) -> Vec<res::PFunction> {
    let mut parser: res::Parser = res::Parser::new(opt, tokens);
    parser.toplevel();

    parser.give_functions()
}

impl<'a> res::Parser<'a> {
    fn toplevel(&mut self) {
        self.skip_require();

        // 関数列のパース
        loop {
            if !self.cur_token_is(&res::TokenKind::FUNC) {
                break;
            }

            self.function();
        }
    }

    fn function(&mut self) {
        let def_func_pos = self.current_token().get_pos();
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

        let mut defined_func = res::PFunction::new(name, return_type, args, def_func_pos);
        defined_func.set_locals(arg_map);
        self.add_pfunction(defined_func);

        let statements = self.compound_statement();

        self.replace_statements(statements);
    }

    pub fn expect_ptype(&mut self) -> res::PType {
        let ptype_offset = self.save_current_offset();
        self.progress();

        let ptype_kind = self.get_specified_token(ptype_offset);
        match *ptype_kind {
            res::TokenKind::INT64 => res::PType::new_int64(),
            res::TokenKind::NORETURN => res::PType::new_noreturn(),
            res::TokenKind::STR => res::PType::new_str(),
            res::TokenKind::BOOLEAN => res::PType::new_boolean(),
            _ => panic!("can't find such a type -> {}", ptype_kind.to_str()),
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
