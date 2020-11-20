use super::ast;
use typed_arena::Arena;
/// フロントエンド資源アロケータ
pub struct Allocator<'a> {
    pub expr_arena: Arena<ast::ExprInfo<'a>>,
    pub stmt_arena: Arena<ast::ExprInfo<'a>>,
}

impl<'a> Default for Allocator<'a> {
    fn default() -> Self {
        Self {
            expr_arena: Arena::new(),
            stmt_arena: Arena::new(),
        }
    }
}
