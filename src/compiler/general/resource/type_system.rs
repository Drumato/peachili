use crate::common::{error as er, option};

pub struct TypeChecker<'a> {
    _build_option: &'a option::BuildOption,
    errors: Vec<er::CompileError>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(build_opt: &'a option::BuildOption) -> Self {
        Self {
            _build_option: build_opt,
            errors: Vec::new(),
        }
    }

    pub fn give_errors(self) -> Vec<er::CompileError> {
        self.errors
    }
    pub fn detect_error(&mut self, e: er::CompileError) {
        self.errors.push(e);
    }
}
