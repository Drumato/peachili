use crate::compiler::general::resource as res;

pub struct BundleParser<'a> {
    tokens: &'a mut Vec<res::Token>,
}

impl<'a> BundleParser<'a> {
    pub fn new(tks: &'a mut Vec<res::Token>) -> Self {
        Self { tokens: tks }
    }

    pub fn require_found(&self) -> bool {
        if self.tokens_invalid() {
            panic!("bundle failed in parsing file.");
        }
        self.tokens[0].kind == res::TokenKind::REQUIRE
    }
    pub fn parse_each_modules(&mut self) -> Vec<res::PStringId> {
        if self.tokens[1].kind != res::TokenKind::LPAREN {
            panic!("enumerating required modules must be start with `(`");
        }

        let mut module_offset: usize = 2;
        let mut reqs_name: Vec<res::PStringId> = Vec::new();

        loop {
            if self.tokens[module_offset].kind == res::TokenKind::RPAREN {
                break;
            }

            let str_id = self.tokens[module_offset].get_str_id();
            if str_id.is_none() {
                panic!(
                    "{} module name must be covered with `\"`",
                    self.tokens[module_offset].get_pos()
                );
            }

            reqs_name.push(str_id.unwrap());
            module_offset += 1;
        }

        reqs_name
    }

    fn tokens_invalid(&self) -> bool {
        self.tokens.is_empty()
    }
}
