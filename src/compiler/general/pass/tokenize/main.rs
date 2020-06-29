use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::common::{error, module, operate, option};
use crate::compiler::general::resource as res;

type Tokens = Vec<res::Token>;
type GuardedAllocator = Arc<Mutex<res::ConstAllocator>>;
type TokenizeReturn = (Tokens, GuardedAllocator);
type CompileErrors = Vec<error::CompileError>;

pub fn tokenize_phase(
    build_option: &option::BuildOption,
    module: &mut module::Module,
    contents: String,
    const_pool: Arc<Mutex<res::ConstAllocator>>,
    buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
) -> TokenizeReturn {
    let ((tokens, const_allocator), tokenize_errors) =
        tokenize(build_option, contents, const_pool, buffer_cache);
    if !tokenize_errors.is_empty() {
        operate::emit_all_errors_and_exit(&tokenize_errors, &module.file_path, build_option);
    }

    if build_option.verbose {
        eprintln!("++++++++ tokenize ++++++++");
        for tk in tokens.iter() {
            eprintln!("\t{}", tk);
        }
    }

    (tokens, const_allocator)
}

fn tokenize(
    opt: &option::BuildOption,
    contents: String,
    const_pool: Arc<Mutex<res::ConstAllocator>>,
    buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
) -> (TokenizeReturn, CompileErrors) {
    let mut lexer = res::Lexer::new(opt, contents, const_pool, buffer_cache);

    lexer.construct_tokens();

    // lexer is dropped after calling `lexer.give_token()`.
    let errors = lexer.copy_errors();
    (lexer.give_token_and_const_arena(), errors)
}

impl<'a> res::Lexer<'a> {
    fn construct_tokens(&mut self) {
        while let Some(t) = self.scan() {
            if t.should_ignore() {
                continue;
            }

            self.add_token(t);

            if let res::TokenKind::EOF = self.last_token().kind {
                break;
            }
        }
    }

    fn scan(&mut self) -> Option<res::Token> {
        if self.offset_overruns_contents_length() {
            let cur_pos = self.cur_position();
            return Some(res::Token::new(cur_pos, res::TokenKind::EOF));
        }

        let look_char = self.cur_offset_char();
        match look_char {
            // 整数トークン
            number if number.is_ascii_digit() => self.scan_number(),

            // 文字列リテラル
            '"' => self.scan_strlit(),

            // 識別子
            c if c.is_ascii_alphabetic() => self.scan_word(),
            '_' => self.scan_word(),

            // 空白類文字
            ' ' | '\t' => self.skip_whitespace(),
            '\n' => {
                let cur_pos = self.cur_position();
                self.skip_crlf();
                Some(res::Token::new(cur_pos, res::TokenKind::NEWLINE))
            }

            _ => {
                let symbol = self.scan_symbol();
                if let Some(tk) = &symbol {
                    if let res::TokenKind::DOUBLESLASH = tk.kind {
                        return self.skip_comment();
                    }
                }

                symbol
            }
        }
    }

    fn scan_strlit(&mut self) -> Option<res::Token> {
        let cur_pos = self.cur_position();

        self.skip_offset(1);

        let contents = self.cut_contents(|c| c != &'"');
        let cached = self.check_already_alloced(&contents);

        // 終端の `"` を読み飛ばすため+1
        self.skip_offset(contents.len() + 1);

        let string_lit_id = if cached {
            self.get_cached_id(&contents)
        } else {
            self.insert_new_buffer(contents)
        };

        Some(res::Token::new(
            cur_pos,
            res::TokenKind::STRLIT(string_lit_id),
        ))
    }
    fn scan_word(&mut self) -> Option<res::Token> {
        // 現在のオフセットを退避
        let cur_pos = self.cur_position();

        // 文字列を読み取る
        let word = self.cut_contents(|c| c.is_alphabetic() || c == &'_' || c.is_ascii_digit());
        let cached = self.check_already_alloced(&word);

        // オフセットを進める
        self.skip_offset(word.len());

        // 予約語かチェック
        if let Some(t_kind) = Self::check_reserved(&word) {
            return Some(res::Token::new(cur_pos, t_kind));
        }

        // 識別子
        let ident_id = if cached {
            self.get_cached_id(&word)
        } else {
            self.insert_new_buffer(word)
        };

        Some(res::Token::new(
            cur_pos,
            res::TokenKind::IDENTIFIER(ident_id),
        ))
    }

    fn scan_number(&mut self) -> Option<res::Token> {
        let number_str = self.cut_contents(|c| c.is_ascii_digit());
        let length = number_str.len();
        let decimal_value = number_str.parse::<i64>();

        let cur_pos = self.cur_position();
        if decimal_value.is_err() {
            let err = error::CompileError::out_of_64bit_sint_range(number_str, cur_pos);
            self.detect_error(err);

            return None;
        }

        self.skip_offset(length);

        if self.cur_offset_char() == 'u' {
            let unsigned_value = number_str.parse::<u64>().unwrap();
            self.skip_offset(1);

            return Some(res::Token::new_uint(cur_pos, unsigned_value));
        }

        Some(res::Token::new_int(cur_pos, decimal_value.unwrap()))
    }

    fn scan_symbol(&mut self) -> Option<res::Token> {
        let multilength_symbols = vec!["::", "//"];

        for sym_str in multilength_symbols.iter() {
            if self.contents_starts_with(sym_str) {
                let length = sym_str.len();
                let cur_pos = self.cur_position();
                self.skip_offset(length);

                return Some(res::Token::new(
                    cur_pos,
                    res::TokenKind::new_from_string(sym_str),
                ));
            }
        }

        let simple_symbols = vec!["+", "-", "*", "/", ";", "(", ")", "{", "}", "=", ",", "&"];

        for sym_str in simple_symbols.iter() {
            if self.contents_starts_with(sym_str) {
                let cur_pos = self.cur_position();
                self.skip_offset(1);

                return Some(res::Token::new(
                    cur_pos,
                    res::TokenKind::new_from_string(sym_str),
                ));
            }
        }

        None
    }

    fn skip_whitespace(&mut self) -> Option<res::Token> {
        let cur_pos = self.cur_position();
        let ws = self.cut_contents(|c| c.is_whitespace() || c == &'\t');
        self.skip_offset(ws.len());

        Some(res::Token::new(cur_pos, res::TokenKind::BLANK))
    }

    fn skip_comment(&mut self) -> Option<res::Token> {
        let cur_pos = self.cur_position();

        // 改行+1まで飛ばす
        let ws = self.cut_contents(|c| c != &'\n');
        self.skip_offset(ws.len());

        let mut newline_pos = self.cur_position();
        newline_pos.add_row(1);
        newline_pos.set_col(1);

        self.set_position(newline_pos);
        self.consume_contents(1);

        Some(res::Token::new(cur_pos, res::TokenKind::BLANK))
    }

    fn check_reserved(s: &str) -> Option<res::TokenKind> {
        match s {
            "true" => Some(res::TokenKind::TRUE),
            "false" => Some(res::TokenKind::FALSE),
            "Boolean" => Some(res::TokenKind::BOOLEAN),
            "ifret" => Some(res::TokenKind::IFRET),
            "if" => Some(res::TokenKind::IF),
            "else" => Some(res::TokenKind::ELSE),
            "Int64" => Some(res::TokenKind::INT64),
            "func" => Some(res::TokenKind::FUNC),
            "return" => Some(res::TokenKind::RETURN),
            "declare" => Some(res::TokenKind::DECLARE),
            "countup" => Some(res::TokenKind::COUNTUP),
            "from" => Some(res::TokenKind::FROM),
            "to" => Some(res::TokenKind::TO),
            "require" => Some(res::TokenKind::REQUIRE),
            "asm" => Some(res::TokenKind::ASM),
            "Noreturn" => Some(res::TokenKind::NORETURN),
            "ConstStr" => Some(res::TokenKind::CONSTSTR),
            "pubtype" => Some(res::TokenKind::PUBTYPE),
            "Uint64" => Some(res::TokenKind::UINT64),
            "varinit" => Some(res::TokenKind::VARINIT),
            "const" => Some(res::TokenKind::CONST),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tokenize_main_tests {
    use super::*;

    #[test]
    fn test_check_reserved() {
        assert!(res::Lexer::check_reserved("true").is_some());
        assert!(res::Lexer::check_reserved("false").is_some());
        assert!(res::Lexer::check_reserved("boolean").is_some());
        assert!(res::Lexer::check_reserved("ifret").is_some());
        assert!(res::Lexer::check_reserved("if").is_some());
        assert!(res::Lexer::check_reserved("else").is_some());
        assert!(res::Lexer::check_reserved("int64").is_some());
        assert!(res::Lexer::check_reserved("func").is_some());
        assert!(res::Lexer::check_reserved("return").is_some());
        assert!(res::Lexer::check_reserved("declare").is_some());
        assert!(res::Lexer::check_reserved("countup").is_some());
        assert!(res::Lexer::check_reserved("from").is_some());
        assert!(res::Lexer::check_reserved("to").is_some());
        assert!(res::Lexer::check_reserved("require").is_some());
        assert!(res::Lexer::check_reserved("asm").is_some());
        assert!(res::Lexer::check_reserved("noreturn").is_some());
        assert!(res::Lexer::check_reserved("str").is_some());
        assert!(res::Lexer::check_reserved("pubtype").is_some());
        assert!(res::Lexer::check_reserved("uint64").is_some());
        assert!(res::Lexer::check_reserved("varinit").is_some());
        assert!(res::Lexer::check_reserved("const").is_some());

        assert!(res::Lexer::check_reserved("ident").is_none());
        assert!(res::Lexer::check_reserved("x_value").is_none());
        assert!(res::Lexer::check_reserved("x11").is_none());
    }
}
