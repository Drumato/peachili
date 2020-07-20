use crate::common::ast::*;
use crate::common::token::{Token, TokenKind};

use crate::common::compiler::parser::*;
use std::collections::BTreeMap;

use id_arena::Arena;
use std::sync::{Arc, Mutex};

pub fn main(fn_arena: FnArena, mut tokens: Vec<Token>, module_name: String) -> ASTRoot {
    let mut ast_root: ASTRoot = Default::default();

    // program -> toplevel*
    loop {
        let t = parser_util::head(&tokens);

        match t.get_kind() {
            TokenKind::FUNC => {
                let (fn_id, rest_tokens) = func_def(fn_arena.clone(), module_name.clone(), tokens);
                tokens = rest_tokens;

                ast_root.funcs.push(fn_id);
            }
            TokenKind::STRUCT => {
                let (type_name, struct_def, rest_tokens) = struct_def(module_name.clone(), tokens);
                tokens = rest_tokens;

                ast_root
                    .typedefs
                    .insert(format!("{}::{}", module_name, type_name), struct_def);
            }
            TokenKind::PUBTYPE => {
                let (alias_name, src_name, rest_tokens) = type_alias(module_name.clone(), tokens);
                tokens = rest_tokens;

                ast_root
                    .alias
                    .insert(format!("{}::{}", module_name, alias_name), src_name);
            }
            _ => break,
        }
    }

    ast_root
}

fn struct_def(module_name: String, mut tokens: Vec<Token>) -> (String, StructDef, Vec<Token>) {
    parser_util::eat_token(&mut tokens);

    let (type_names, rest_tokens) = parser_util::expect_identifier(tokens);
    let type_name = type_names[0].clone();

    let (members, rest_tokens) = member_block(module_name, rest_tokens);
    (type_name, StructDef { members }, rest_tokens)
}

fn member_block(module_name: String, mut tokens: Vec<Token>) -> (BTreeMap<String, String>, Vec<Token>) {
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

        let (member_type, rest_tokens) = parser_util::expect_type(module_name.clone(), tokens);
        tokens = rest_tokens;

        members.insert(member_name, member_type);
    }

    (members, tokens)
}

fn type_alias(module_name: String, mut tokens: Vec<Token>) -> (String, String, Vec<Token>) {
    parser_util::eat_token(&mut tokens);

    let (alias_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
    let alias_name = alias_names[0].clone();

    parser_util::expect(TokenKind::ASSIGN, &mut rest_tokens);

    let (src_name, mut rest_tokens) = parser_util::expect_type(module_name, rest_tokens);
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (alias_name, src_name, rest_tokens)
}

fn func_def(fn_arena: FnArena, module_name: String, mut tokens: Vec<Token>) -> (FnId, Vec<Token>) {
    let func_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);

    let stmt_arena = Arc::new(Mutex::new(Arena::new()));
    let expr_arena = Arc::new(Mutex::new(Arena::new()));

    let (func_names, rest_tokens) = parser_util::expect_identifier(tokens);
    let func_name = func_names[0].clone();

    let (arg_map, rest_tokens) = arg_list(module_name.clone(), rest_tokens);

    let (return_type, rest_tokens) = parser_util::expect_type(module_name.clone(), rest_tokens);

    let (stmts, rest_tokens) =
        parser_util::expect_block(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), rest_tokens);

    (
        fn_arena.lock().unwrap().alloc(Function {
            name: func_name,
            args: arg_map,
            stmts,
            return_type,
            pos: func_pos,
            module_name,
            stmt_arena,
            expr_arena,
        }),
        rest_tokens,
    )
}

fn arg_list(module_name: String, mut tokens: Vec<Token>) -> (BTreeMap<String, String>, Vec<Token>) {
    parser_util::expect(TokenKind::LPAREN, &mut tokens);

    let mut arg_map = BTreeMap::new();

    loop {
        let t = parser_util::head(&tokens);

        if t.get_kind() == &TokenKind::RPAREN {
            parser_util::expect(TokenKind::RPAREN, &mut tokens);
            break;
        }

        let (arg_names, rest_tokens) = parser_util::expect_identifier(tokens);
        let arg_name = arg_names[0].clone();
        tokens = rest_tokens;

        let (type_name, rest_tokens) = parser_util::expect_type(module_name.clone(), tokens);
        tokens = rest_tokens;

        parser_util::consume(TokenKind::COMMA, &mut tokens);

        arg_map.insert(arg_name, type_name);
    }

    (arg_map, tokens)
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;

    use id_arena::Arena;
    use std::sync::{Arc, Mutex};

    #[test]
    fn type_alias_test() {
        let tokens = vec![
            Token::new(TokenKind::PUBTYPE, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::ASSIGN, Default::default()),
            Token::new_identifier("Fizz".to_string(), Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        let (alias_name, src_name, rest_tokens) = type_alias(Default::default(), tokens);

        assert_eq!("foo", alias_name);
        assert_eq!("::Fizz", src_name);
        assert_eq!(1, rest_tokens.len());
    }

    #[test]
    fn struct_def_test() {
        let tokens = vec![
            Token::new(TokenKind::STRUCT, Default::default()),
            Token::new_identifier("X".to_string(), Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new_identifier("bar".to_string(), Default::default()),
            Token::new(TokenKind::UINT64, Default::default()),
            Token::new_identifier("fizz".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        let (type_name, struct_def, rest_tokens) = struct_def(Default::default(), tokens);

        assert_eq!("X", type_name);
        assert_eq!(3, struct_def.members.len());
        assert_eq!(1, rest_tokens.len());
    }

    #[test]
    fn member_block_test() {
        let tokens = vec![
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new_identifier("bar".to_string(), Default::default()),
            Token::new(TokenKind::UINT64, Default::default()),
            Token::new_identifier("fizz".to_string(), Default::default()),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        let (member_map, rest_tokens) = member_block(Default::default(), tokens);

        assert_eq!(3, member_map.len());
        assert_eq!(1, rest_tokens.len());
    }

    #[test]
    fn arg_list_test() {
        let tokens = vec![
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
            Token::new(TokenKind::EOF, Default::default()),
        ];
        let (arg_map, rest_tokens) = arg_list(Default::default(), tokens);

        assert_eq!(3, arg_map.len());
        assert_eq!(1, rest_tokens.len());
    }

    #[test]
    fn func_def_test() {
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
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let fn_arena = new_allocators();

        let (_fn_id, rest_tokens) = func_def(fn_arena, "sample".to_string(), tokens);

        assert_eq!(1, rest_tokens.len());
    }

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
