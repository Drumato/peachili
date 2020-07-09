use crate::common::ast::*;
use crate::common::token::{Token, TokenKind};

use indexmap::IndexMap;
use crate::common::compiler::parser::*;

use std::sync::{Arc, Mutex};

use id_arena::Arena;
use crate::common::compiler::parser::parser_util::expect_type;

type FnArena = Arc<Mutex<Arena<Function>>>;
type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;


pub fn main(fn_arena: FnArena, stmt_arena: StmtArena, expr_arena: ExprArena, mut tokens: Vec<Token>, parent_modules: Vec<String>) -> ASTRoot {
    let mut ast_root: ASTRoot = Default::default();
    ast_root.parent_modules = parent_modules;

    // program -> toplevel*
    loop {
        let t = parser_util::head(&tokens);

        match t.get_kind() {
            TokenKind::FUNC => {
                let (fn_id, rest_tokens) = func_def(fn_arena.clone(), stmt_arena.clone(), expr_arena.clone(), tokens);
                tokens = rest_tokens;

                ast_root.funcs.push(fn_id);
            }
            TokenKind::STRUCT => {
                let (type_name, struct_def, rest_tokens) = struct_def(tokens);
                tokens = rest_tokens;

                ast_root.typedefs.insert(type_name, struct_def);
            }
            TokenKind::PUBTYPE => {
                let (alias_name, src_name, rest_tokens) = type_alias(tokens);
                tokens = rest_tokens;

                ast_root.alias.insert(alias_name, src_name);
            }
            _ => break,
        }
        if t.get_kind() == &TokenKind::EOF {
            break;
        }
    }

    ast_root
}

fn struct_def(mut tokens: Vec<Token>) -> (String, StructDef, Vec<Token>) {
    parser_util::eat_token(&mut tokens);

    let (type_names, rest_tokens) = parser_util::expect_identifier(tokens);
    let type_name = type_names[0].clone();

    let (members, rest_tokens) = member_block(rest_tokens);
    (type_name, StructDef { members }, rest_tokens)
}

fn member_block(mut tokens: Vec<Token>) -> (IndexMap<String, String>, Vec<Token>) {
    let mut members = IndexMap::new();
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

        let (member_type, rest_tokens) = parser_util::expect_type(tokens);
        tokens = rest_tokens;

        members.insert(member_name, member_type);
    }

    (members, tokens)
}

fn type_alias(mut tokens: Vec<Token>) -> (String, String, Vec<Token>) {
    parser_util::eat_token(&mut tokens);

    let (alias_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
    let alias_name = alias_names[0].clone();

    parser_util::expect(TokenKind::ASSIGN, &mut rest_tokens);

    let (src_name, mut rest_tokens) = expect_type(rest_tokens);
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (alias_name, src_name, rest_tokens)
}

fn func_def(fn_arena: FnArena, stmt_arena: StmtArena, expr_arena: ExprArena, mut tokens: Vec<Token>) -> (FnId, Vec<Token>) {
    let func_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);

    let (func_names, rest_tokens) = parser_util::expect_identifier(tokens);
    let func_name = func_names[0].clone();

    let (arg_map, rest_tokens) = arg_list(rest_tokens);

    let (return_type, rest_tokens) = parser_util::expect_type(rest_tokens);

    let (stmts, rest_tokens) = parser_util::expect_block(stmt_arena, expr_arena, rest_tokens);

    (fn_arena.lock().unwrap().alloc(
        Function {
            name: func_name,
            args: arg_map,
            stmts,
            return_type,
            pos: func_pos,
        }
    ), rest_tokens)
}

fn arg_list(mut tokens: Vec<Token>) -> (IndexMap<String, String>, Vec<Token>) {
    parser_util::expect(TokenKind::LPAREN, &mut tokens);

    let mut arg_map = IndexMap::new();

    loop {
        let t = parser_util::head(&tokens);

        if t.get_kind() == &TokenKind::RPAREN {
            parser_util::expect(TokenKind::RPAREN, &mut tokens);
            break;
        }
        let (arg_names, rest_tokens) = parser_util::expect_identifier(tokens);
        let arg_name = arg_names[0].clone();
        tokens = rest_tokens;

        let (type_name, rest_tokens) = parser_util::expect_type(tokens);
        tokens = rest_tokens;

        parser_util::consume(TokenKind::COMMA, &mut tokens);

        arg_map.insert(arg_name, type_name);
    }

    (arg_map, tokens)
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;

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
        let (alias_name, src_name, rest_tokens) = type_alias(tokens);

        assert_eq!("foo", alias_name);
        assert_eq!("Fizz", src_name);
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
        let (type_name, struct_def, rest_tokens) = struct_def(tokens);

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
        let (member_map, rest_tokens) = member_block(tokens);

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
        let (arg_map, rest_tokens) = arg_list(tokens);

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

        let (fn_arena, stmt_arena, expr_arena) = new_allocators();

        let (_fn_id, rest_tokens) = func_def(fn_arena, stmt_arena, expr_arena, tokens);

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

        let (fn_arena, stmt_arena, expr_arena) = new_allocators();

        let root = main(fn_arena, stmt_arena, expr_arena, tokens, Vec::new());

        assert_eq!(2, root.funcs.len());
    }

    fn new_allocators() -> (
        FnArena,
        StmtArena,
        ExprArena,
    ) {
        (
            Arc::new(Mutex::new(Arena::new())),
            Arc::new(Mutex::new(Arena::new())),
            Arc::new(Mutex::new(Arena::new())),
        )
    }
}