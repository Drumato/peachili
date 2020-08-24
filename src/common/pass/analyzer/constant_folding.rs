use crate::common::analyze_resource::ast;

/// ASTに対する定数畳み込みのメインルーチン
pub fn constant_folding(fn_arena: ast::FnArena, full_ast: &ast::ASTRoot) {
    for fn_id in full_ast.funcs.iter() {
        if let Ok(ref mut arena) = fn_arena.lock() {
            if let Some(ast_fn) = arena.get_mut(*fn_id) {
                folding_fn(ast_fn);
            }
        }
    }
}

fn folding_fn(ast_fn: &mut ast::Function) {
    if let Ok(ref mut stmt_arena) = ast_fn.stmt_arena.lock() {
        for stmt_id in ast_fn.stmts.iter() {
            if let Some(stmt) = stmt_arena.get_mut(*stmt_id) {
                *stmt = folding_stmt(ast_fn.expr_arena.clone(), stmt);
            }
        }
    }
}

fn folding_stmt(expr_arena: ast::ExprArena, ast_stmt: &ast::StatementNode) -> ast::StatementNode {
    match ast_stmt.get_kind() {
        ast::StatementNodeKind::VARINIT {
            ident_name,
            type_name,
            expr: expr_id,
        } => {
            let initialize_expr = folding_expr(expr_arena.clone(), *expr_id);
            ast::StatementNode::new(
                ast::StatementNodeKind::VARINIT {
                    ident_name: ident_name.clone(),
                    type_name: type_name.clone(),
                    expr: expr_arena.lock().unwrap().alloc(initialize_expr),
                },
                ast_stmt.get_position(),
            )
        }
        ast::StatementNodeKind::RETURN { expr: expr_id } => {
            let return_expr = folding_expr(expr_arena.clone(), *expr_id);

            ast::StatementNode::new(
                ast::StatementNodeKind::RETURN {
                    expr: expr_arena.lock().unwrap().alloc(return_expr),
                },
                ast_stmt.get_position(),
            )
        }
        ast::StatementNodeKind::EXPR { expr: expr_id } => {
            let expr = folding_expr(expr_arena.clone(), *expr_id);

            ast::StatementNode::new(
                ast::StatementNodeKind::EXPR {
                    expr: expr_arena.lock().unwrap().alloc(expr),
                },
                ast_stmt.get_position(),
            )
        }
        _ => ast_stmt.clone(),
    }
}

fn folding_expr(expr_arena: ast::ExprArena, ast_expr_id: ast::ExNodeId) -> ast::ExpressionNode {
    let ast_expr = expr_arena.lock().unwrap().get(ast_expr_id).unwrap().clone();
    match ast_expr.get_kind() {
        ast::ExpressionNodeKind::ADD {
            lhs: lhs_id,
            rhs: rhs_id,
        } => {
            let lhs = folding_expr(expr_arena.clone(), *lhs_id);
            let rhs = folding_expr(expr_arena.clone(), *rhs_id);

            if lhs.is_integer_literal() && rhs.is_integer_literal() {
                return ast::ExpressionNode::new_integer(
                    lhs.get_integer_value() + rhs.get_integer_value(),
                    lhs.get_pos(),
                );
            }

            ast_expr
        }
        ast::ExpressionNodeKind::SUB {
            lhs: lhs_id,
            rhs: rhs_id,
        } => {
            let lhs = folding_expr(expr_arena.clone(), *lhs_id);
            let rhs = folding_expr(expr_arena.clone(), *rhs_id);

            if lhs.is_integer_literal() && rhs.is_integer_literal() {
                return ast::ExpressionNode::new_integer(
                    lhs.get_integer_value() - rhs.get_integer_value(),
                    lhs.get_pos(),
                );
            }

            ast_expr
        }
        ast::ExpressionNodeKind::MUL {
            lhs: lhs_id,
            rhs: rhs_id,
        } => {
            let lhs = folding_expr(expr_arena.clone(), *lhs_id);
            let rhs = folding_expr(expr_arena.clone(), *rhs_id);

            if lhs.is_integer_literal() && rhs.is_integer_literal() {
                return ast::ExpressionNode::new_integer(
                    lhs.get_integer_value() * rhs.get_integer_value(),
                    lhs.get_pos(),
                );
            }

            ast_expr
        }
        ast::ExpressionNodeKind::DIV {
            lhs: lhs_id,
            rhs: rhs_id,
        } => {
            let lhs = folding_expr(expr_arena.clone(), *lhs_id);
            let rhs = folding_expr(expr_arena.clone(), *rhs_id);

            if lhs.is_integer_literal() && rhs.is_integer_literal() {
                return ast::ExpressionNode::new_integer(
                    lhs.get_integer_value() / rhs.get_integer_value(),
                    lhs.get_pos(),
                );
            }

            ast_expr
        }
        ast::ExpressionNodeKind::NEG { value: value_id } => {
            let value = folding_expr(expr_arena.clone(), *value_id);

            if value.is_integer_literal() {
                return ast::ExpressionNode::new_integer(
                    -value.get_integer_value(),
                    value.get_pos(),
                );
            }

            ast_expr
        }
        ast::ExpressionNodeKind::CALL { names, args } => {
            let mut optimized_args: Vec<ast::ExNodeId> = Vec::new();

            for arg_id in args.iter() {
                let optimized_arg = folding_expr(expr_arena.clone(), *arg_id);
                optimized_args.push(expr_arena.lock().unwrap().alloc(optimized_arg));
            }

            ast::ExpressionNode::new_call(names.clone(), optimized_args, ast_expr.get_pos())
        }
        _ => ast_expr,
    }
}
