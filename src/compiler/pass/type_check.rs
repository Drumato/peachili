use std::collections::BTreeMap;

use crate::common::{error as er, option};
use crate::compiler::resource as res;

// 型チェック時に毎回タイプを生成するとコストがかかる
// ここではグローバルな実体の参照を取り回すことで，型検査を実装する
const GLOBAL_INT_TYPE: res::PType = res::PType {
    kind: res::PTypeKind::INT64,
    size: 8,
};
const GLOBAL_BOOLEAN_TYPE: res::PType = res::PType {
    kind: res::PTypeKind::BOOLEAN,
    size: 8,
};
const GLOBAL_STR_TYPE: res::PType = res::PType {
    kind: res::PTypeKind::STR,
    size: 8,
};
const GLOBAL_NORETURN_TYPE: res::PType = res::PType {
    kind: res::PTypeKind::NORETURN,
    size: 0,
};

pub fn type_check_fn(
    build_opt: &option::BuildOption,
    all_funcs: &BTreeMap<String, res::PFunction>,
    this_func: &res::PFunction,
) -> Vec<er::CompileError> {
    let stmts = this_func.get_statements();
    let locals = this_func.get_locals();

    let mut checker = res::TypeChecker::new(build_opt);
    for st in stmts.iter() {
        checker.check_statement(st, all_funcs, locals);
    }

    checker.give_errors()
}

impl<'a> res::TypeChecker<'a> {
    fn check_statement(
        &mut self,
        st: &res::StatementNode,
        all_funcs: &BTreeMap<String, res::PFunction>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        match &st.kind {
            res::StatementNodeKind::RETURN(_return_expr) => None,
            res::StatementNodeKind::IFRET(return_expr) => {
                self.check_expression(return_expr, all_funcs, locals)
            }
            res::StatementNodeKind::EXPR(expr) => self.check_expression(expr, all_funcs, locals),
            res::StatementNodeKind::VARDECL => None,
            res::StatementNodeKind::COUNTUP(_ident, _start_expr, _end_expr, _body) => None,
            res::StatementNodeKind::ASM(_asm_literals) => None,
        }
    }

    fn check_expression(
        &mut self,
        ex: &res::ExpressionNode,
        all_funcs: &BTreeMap<String, res::PFunction>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        match &ex.kind {
            res::ExpressionNodeKind::INTEGER(_v) => Some(GLOBAL_INT_TYPE),
            res::ExpressionNodeKind::STRLIT(_contents, _hash) => Some(GLOBAL_STR_TYPE),
            res::ExpressionNodeKind::TRUE | res::ExpressionNodeKind::FALSE => {
                Some(GLOBAL_BOOLEAN_TYPE)
            }
            res::ExpressionNodeKind::IDENT(name) => {
                let defined_name = res::IdentName::last_name(name);

                if locals.get(&defined_name).is_none() {
                    let ident_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::used_undefined_auto_var(
                        defined_name,
                        ident_pos,
                    ));

                    return None;
                }

                let local_var = locals.get(&defined_name).unwrap();
                Some(self.get_global_type_from(local_var.get_type()))
            }

            res::ExpressionNodeKind::CALL(name, _args) => {
                let called_name = res::IdentName::last_name(name);

                let called_func_opt = all_funcs.get(&called_name);

                if called_func_opt.is_none() {
                    let call_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::called_undefined_function(
                        called_name,
                        call_pos,
                    ));

                    return None;
                }
                Some(self.get_global_type_from(called_func_opt.unwrap().get_return_type()))
            }

            res::ExpressionNodeKind::ADD(lop, rop)
            | res::ExpressionNodeKind::SUB(lop, rop)
            | res::ExpressionNodeKind::MUL(lop, rop)
            | res::ExpressionNodeKind::DIV(lop, rop) => {
                let lop_type = self.try_to_resolve_expression(lop, all_funcs, locals);
                let rop_type = self.try_to_resolve_expression(rop, all_funcs, locals);

                if lop_type.is_none() || rop_type.is_none() {
                    return None;
                }

                if lop_type != rop_type {
                    let err_pos = ex.copy_pos();
                    self.detect_error(
                        er::CompileError::binary_operation_must_have_two_same_type_operands(
                            ex.operator_to_string(),
                            lop_type.unwrap(),
                            rop_type.unwrap(),
                            err_pos,
                        ),
                    );
                    return None;
                }

                return lop_type;
            }

            res::ExpressionNodeKind::ASSIGN(lvalue, rvalue) => {
                let lvalue_type = self.try_to_resolve_expression(lvalue, all_funcs, locals);
                let rvalue_type = self.try_to_resolve_expression(rvalue, all_funcs, locals);

                // try_to_resolve_expression() とは別にエラーを生成
                if rvalue_type.is_none() {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::cannot_assignment_unresolved_right_value(
                        *lvalue.clone(),
                        *rvalue.clone(),
                        err_pos,
                    ));
                    return None;
                }

                if lvalue_type != rvalue_type {
                    let err_pos = ex.copy_pos();
                    self.detect_error(
                        er::CompileError::both_values_must_be_same_type_in_assignment(
                            lvalue_type.unwrap(),
                            rvalue_type.unwrap(),
                            err_pos,
                        ),
                    );
                    return None;
                }

                rvalue_type
            }

            res::ExpressionNodeKind::IF(cond_expr, body) => {
                if self.detect_conditional_expression_error(cond_expr, all_funcs, locals) {
                    return None;
                }
                self.check_block_statement(body, all_funcs, locals)
            }
            res::ExpressionNodeKind::IFELSE(cond_expr, body, alter) => {
                if self.detect_conditional_expression_error(cond_expr, all_funcs, locals) {
                    return None;
                }

                let body_type = self.check_block_statement(body, all_funcs, locals);
                let alter_type = self.check_block_statement(alter, all_funcs, locals);

                if body_type != alter_type {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::both_blocks_must_be_same_type(err_pos));

                    return None;
                }

                body_type
            }
            _ => None,
        }
    }

    /// エラーが見つかれば格納してtrue
    /// そうでなければfalse
    fn detect_conditional_expression_error(
        &mut self,
        cond_expr: &res::ExpressionNode,
        all_funcs: &BTreeMap<String, res::PFunction>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> bool {
        // 型が解決できるかチェック
        let cond_expr_type = self.try_to_resolve_expression(cond_expr, all_funcs, locals);
        if cond_expr_type.is_none() {
            return true;
        }

        // boolean型かチェック
        let cond_expr_type = cond_expr_type.unwrap();
        if !self.is_boolean_type(&cond_expr_type) {
            let err_pos = cond_expr.copy_pos();
            self.detect_error(
                er::CompileError::condition_expression_must_be_an_boolean_in_if(
                    cond_expr_type,
                    err_pos,
                ),
            );

            return true;
        }

        false
    }

    fn check_block_statement(
        &mut self,
        block: &[res::StatementNode],
        all_funcs: &BTreeMap<String, res::PFunction>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        for st in block.iter() {
            if !self.is_ifret(st) {
                continue;
            }

            // ifret-statementのチェック
            return self.check_statement(st, all_funcs, locals);
        }

        None
    }

    fn try_to_resolve_expression(
        &mut self,
        ex: &res::ExpressionNode,
        all_funcs: &BTreeMap<String, res::PFunction>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        let ex_type = self.check_expression(ex, all_funcs, locals);

        if ex_type.is_none() {
            let err_pos = ex.copy_pos();
            self.detect_error(er::CompileError::unable_to_resolve_expression_type(
                ex.clone(),
                err_pos,
            ));
        }
        ex_type
    }

    fn is_boolean_type(&mut self, t: &res::PType) -> bool {
        match t.kind {
            res::PTypeKind::BOOLEAN => true,
            _ => false,
        }
    }

    fn is_ifret(&mut self, st: &res::StatementNode) -> bool {
        match &st.kind {
            res::StatementNodeKind::IFRET(_expr) => true,
            _ => false,
        }
    }

    fn get_global_type_from(&self, t: &res::PType) -> res::PType {
        match &t.kind {
            res::PTypeKind::BOOLEAN => GLOBAL_BOOLEAN_TYPE,
            res::PTypeKind::INT64 => GLOBAL_INT_TYPE,
            res::PTypeKind::STR => GLOBAL_STR_TYPE,
            res::PTypeKind::NORETURN => GLOBAL_NORETURN_TYPE,
        }
    }
}
