use crate::common::ast::{ASTRoot, FnArena, FnId, Function, FunctionTypeDef, StructDef};
use crate::common::token::{Token, TokenKind};

use crate::common::pass::parser::context::Context;
use crate::common::pass::parser::*;
use id_arena::Arena;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub fn main(fn_arena: FnArena, mut tokens: Vec<Token>, module_name: String) -> ASTRoot {
    let mut ast_root: ASTRoot = Default::default();
    let mut ctxt: Context = Default::default();
    ctxt.fn_arena = fn_arena;
    ctxt.module_name = module_name;

    // program -> toplevel*
    loop {
        let t = parser_util::head(&tokens);

        match t.get_kind() {
            TokenKind::IMPORT => skip_import_directive(&mut tokens),
            TokenKind::FUNC => {
                let (fn_id, rest_tokens) = ctxt.func_def(tokens);
                tokens = rest_tokens;
                ast_root.funcs.push(fn_id);
            }
            TokenKind::STRUCT => {
                let (type_name, struct_def, rest_tokens) = ctxt.struct_def(tokens);
                tokens = rest_tokens;

                ast_root
                    .typedefs
                    .insert(format!("{}::{}", ctxt.module_name, type_name), struct_def);
            }
            TokenKind::PUBTYPE => {
                let (alias_name, src_name, rest_tokens) = ctxt.type_alias(tokens);
                tokens = rest_tokens;

                ast_root
                    .alias
                    .insert(format!("{}::{}", ctxt.module_name, alias_name), src_name);
            }
            _ => break,
        }
    }

    ast_root.called_functions = ctxt.called_functions;
    ast_root
}

impl Context {
    /// 関数定義をパースする関数
    fn func_def(&mut self, mut tokens: Vec<Token>) -> (FnId, Vec<Token>) {
        // 関数ごとにStmt/ExprArenaは初期化する
        self.expr_arena = Arc::new(Mutex::new(Arena::new()));
        self.stmt_arena = Arc::new(Mutex::new(Arena::new()));

        let func_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (func_names, rest_tokens) = parser_util::expect_identifier(tokens);
        let func_name = func_names[0].clone();

        let (arg_map, rest_tokens) = self.arg_list(rest_tokens);

        let (return_type, rest_tokens) = self.expect_type(rest_tokens);

        let (stmts, rest_tokens) = self.expect_block(rest_tokens);

        (
            self.fn_arena.lock().unwrap().alloc(Function {
                name: func_name,
                fn_type: FunctionTypeDef::new(return_type, arg_map),
                stmts,
                pos: func_pos,
                module_name: self.module_name.clone(),
                stmt_arena: self.stmt_arena.clone(),
                expr_arena: self.expr_arena.clone(),
            }),
            rest_tokens,
        )
    }

    /// 引数定義リストをパースする関数
    fn arg_list(&mut self, mut tokens: Vec<Token>) -> (Vec<(String, String)>, Vec<Token>) {
        parser_util::expect(TokenKind::LPAREN, &mut tokens);

        let mut args = Vec::new();

        loop {
            let t = parser_util::head(&tokens);

            if t.get_kind() == &TokenKind::RPAREN {
                parser_util::expect(TokenKind::RPAREN, &mut tokens);
                break;
            }

            let (arg_names, rest_tokens) = parser_util::expect_identifier(tokens);
            let arg_name = arg_names[0].clone();
            tokens = rest_tokens;

            let (type_name, rest_tokens) = self.expect_type(tokens);
            tokens = rest_tokens;

            parser_util::consume(TokenKind::COMMA, &mut tokens);

            args.push((arg_name, type_name));
        }

        (args, tokens)
    }

    /// 構造体型の定義をパースする．
    fn struct_def(&mut self, mut tokens: Vec<Token>) -> (String, StructDef, Vec<Token>) {
        parser_util::eat_token(&mut tokens);

        let (type_names, rest_tokens) = parser_util::expect_identifier(tokens);
        let type_name = type_names[0].clone();

        let (members, rest_tokens) = self.member_block(rest_tokens);
        (type_name, StructDef { members }, rest_tokens)
    }

    /// 構造体型内のメンバ定義列をパースする．
    /// 引数のように，リスト構造をパースするメタ関数を作ってもいいかも．
    fn member_block(&mut self, mut tokens: Vec<Token>) -> (BTreeMap<String, String>, Vec<Token>) {
        let mut members = BTreeMap::new();
        parser_util::expect(TokenKind::LBRACE, &mut tokens);

        loop {
            let t = parser_util::head(&tokens);
            if t.get_kind() == &TokenKind::RBRACE {
                parser_util::expect(TokenKind::RBRACE, &mut tokens);
                break;
            }

            let (member_names, rest_tokens) = parser_util::expect_identifier(tokens);
            tokens = rest_tokens;
            let member_name = member_names[0].clone();

            let (member_type, rest_tokens) = self.expect_type(tokens);
            tokens = rest_tokens;

            members.insert(member_name, member_type);
        }

        (members, tokens)
    }

    /// 型エイリアスをパースする関数
    fn type_alias(&mut self, mut tokens: Vec<Token>) -> (String, String, Vec<Token>) {
        parser_util::eat_token(&mut tokens);

        let (alias_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let alias_name = alias_names[0].clone();

        parser_util::expect(TokenKind::ASSIGN, &mut rest_tokens);

        let (src_name, mut rest_tokens) = self.expect_type(rest_tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (alias_name, src_name, rest_tokens)
    }
}

/// コンパイラ内部では用いないのでスキップする．
/// ASTRootに情報を含めることで，ルートがインポートしていないパッケージはバイナリに含めない，みたいなことができるかも.
fn skip_import_directive(tokens: &mut Vec<Token>) {
    parser_util::eat_token(tokens);
    parser_util::eat_token(tokens);
    parser_util::eat_token(tokens);
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;

    use id_arena::Arena;
    use std::sync::{Arc, Mutex};

    #[test]
    fn type_alias_test() {}

    #[test]
    fn struct_def_test() {}

    #[test]
    fn member_block_test() {}

    #[test]
    fn arg_list_test() {}

    #[test]
    fn func_def_test() {}

    #[test]
    fn main_test() {
        let tokens = vec![
            Token::new(TokenKind::FUNC, Default::default()),
            Token::new_identifier("main".to_string(), Default::default()),
            Token::new(TokenKind::LPAREN, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::COMMA, Default::default()),
            Token::new_identifier("bar".to_string(), Default::default()),
            Token::new(TokenKind::UINT64, Default::default()),
            Token::new(TokenKind::COMMA, Default::default()),
            Token::new_identifier("fizz".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::RPAREN, Default::default()),
            Token::new(TokenKind::NORETURN, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::FUNC, Default::default()),
            Token::new_identifier("sub1".to_string(), Default::default()),
            Token::new(TokenKind::LPAREN, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::COMMA, Default::default()),
            Token::new_identifier("bar".to_string(), Default::default()),
            Token::new(TokenKind::UINT64, Default::default()),
            Token::new(TokenKind::COMMA, Default::default()),
            Token::new_identifier("fizz".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::RPAREN, Default::default()),
            Token::new(TokenKind::NORETURN, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        let fn_arena = new_allocators();

        let root = main(fn_arena, tokens, "sample".to_string());

        assert_eq!(2, root.funcs.len());
    }

    fn new_allocators() -> FnArena {
        Arc::new(Mutex::new(Arena::new()))
    }
}
