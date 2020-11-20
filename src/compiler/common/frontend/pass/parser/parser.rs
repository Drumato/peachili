use crate::compiler::common::frontend::{allocator, ast};
pub struct Parser<'a> {
    pub allocator: &'a allocator::Allocator<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(alloc: &'a allocator::Allocator<'a>) -> Self {
        Self { allocator: alloc }
    }
}
