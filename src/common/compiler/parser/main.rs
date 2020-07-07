use crate::common::ast::ExNodeId;
use crate::common::token::Token;

use crate::common::compiler::parser::expression;

use crate::setup;

pub fn main(tokens: Vec<Token>) -> ExNodeId {
    let (ex_id, _rest_tokens) = expression::expression(setup::AST_EXPR_ARENA.clone(), tokens);

    ex_id
}
