#[cfg(test)]
mod tokenizer_tests {
    use crate::common::option as opt;
    use crate::compiler::pass::tokenize::main::*;
    use crate::compiler::resource::token as tok;

    #[test]
    fn test_tokenize_keywords() {
        let build_option: opt::BuildOption = Default::default();
        let contents = keyword_test_case();

        let (tokens, tokenize_errors) = tokenize(&build_option, contents);

        assert!(tokenize_errors.is_empty());

        let expects = keyword_test_expects();

        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(expects[i], t.kind);
        }
    }

    #[test]
    fn test_tokenize_symbols() {
        let build_option: opt::BuildOption = Default::default();
        let contents = symbol_test_case();

        let (tokens, tokenize_errors) = tokenize(&build_option, contents);

        assert!(tokenize_errors.is_empty());

        let expects = symbol_test_expects();

        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(expects[i], t.kind);
        }
    }

    #[test]
    fn test_tokenize_elements() {
        let build_option: opt::BuildOption = Default::default();
        let contents = element_test_case();

        let (tokens, tokenize_errors) = tokenize(&build_option, contents);

        assert!(tokenize_errors.is_empty());

        let expects = element_test_expects();

        for (i, t) in tokens.iter().enumerate() {
            assert_eq!(expects[i], t.kind);
        }
    }

    fn keyword_test_case() -> String {
        "true false boolean require if else ifret func declare countup from to asm return noreturn int64 str".to_string()
    }

    fn keyword_test_expects() -> Vec<tok::TokenKind> {
        vec![
            tok::TokenKind::TRUE,
            tok::TokenKind::FALSE,
            tok::TokenKind::BOOLEAN,
            tok::TokenKind::REQUIRE,
            tok::TokenKind::IF,
            tok::TokenKind::ELSE,
            tok::TokenKind::IFRET,
            tok::TokenKind::FUNC,
            tok::TokenKind::DECLARE,
            tok::TokenKind::COUNTUP,
            tok::TokenKind::FROM,
            tok::TokenKind::TO,
            tok::TokenKind::ASM,
            tok::TokenKind::RETURN,
            tok::TokenKind::NORETURN,
            tok::TokenKind::INT64,
            tok::TokenKind::STR,
            tok::TokenKind::EOF,
        ]
    }

    fn symbol_test_case() -> String {
        "+ - * / ( ) { } : :: ; = , ".to_string()
    }

    fn symbol_test_expects() -> Vec<tok::TokenKind> {
        vec![
            tok::TokenKind::PLUS,
            tok::TokenKind::MINUS,
            tok::TokenKind::ASTERISK,
            tok::TokenKind::SLASH,
            tok::TokenKind::LPAREN,
            tok::TokenKind::RPAREN,
            tok::TokenKind::LBRACE,
            tok::TokenKind::RBRACE,
            tok::TokenKind::COLON,
            tok::TokenKind::DOUBLECOLON,
            tok::TokenKind::SEMICOLON,
            tok::TokenKind::ASSIGN,
            tok::TokenKind::COMMA,
            tok::TokenKind::EOF,
        ]
    }

    fn element_test_case() -> String {
        "10 20 30 x y_s1 \"Hello, World\"".to_string()
    }

    fn element_test_expects() -> Vec<tok::TokenKind> {
        vec![
            tok::TokenKind::INTEGER(10),
            tok::TokenKind::INTEGER(20),
            tok::TokenKind::INTEGER(30),
            tok::TokenKind::IDENTIFIER("x".to_string()),
            tok::TokenKind::IDENTIFIER("y_s1".to_string()),
            tok::TokenKind::STRLIT("Hello, World".to_string()),
            tok::TokenKind::EOF,
        ]
    }
}
