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
    pub fn parse_each_modules(&mut self) -> Vec<String> {
        if self.tokens[1].kind != res::TokenKind::LPAREN {
            panic!("enumerating required modules must be start with `(`");
        }

        let mut module_offset: usize = 2;
        let mut subs_name: Vec<String> = Vec::new();

        loop {
            if self.tokens[module_offset].kind == res::TokenKind::RPAREN {
                break;
            }

            let cur_contents = self.tokens[module_offset].copy_strlit_contents();
            if cur_contents.is_none() {
                panic!(
                    "{} module name must be covered with `\"`",
                    self.tokens[module_offset].get_pos()
                );
            }

            subs_name.push(cur_contents.unwrap());
            module_offset += 1;
        }

        subs_name
    }

    fn tokens_invalid(&self) -> bool {
        self.tokens.is_empty()
    }
}
