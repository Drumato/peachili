use crate::common::ast::StNodeId;
use crate::common::token::Token;

use crate::common::compiler::parser::statement;

use crate::setup;

pub fn main(tokens: Vec<Token>) -> StNodeId {
    let (st_id, _rest_tokens) = statement::statement(
        setup::AST_STMT_ARENA.clone(),
        setup::AST_EXPR_ARENA.clone(),
        tokens,
    );

    st_id
}
