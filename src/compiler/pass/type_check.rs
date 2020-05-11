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
    all_funcs: &[res::PFunction],
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
        all_funcs: &[res::PFunction],
        locals: &BTreeMap<String, res::PVariable>,
    ) {
        match &st.kind {
            res::StatementNodeKind::RETURN(_return_expr) => {}
            res::StatementNodeKind::IFRET(_return_expr) => {}
            res::StatementNodeKind::EXPR(expr) => {
                self.check_expression(expr, all_funcs, locals);
            }
            res::StatementNodeKind::VARDECL => {}
            res::StatementNodeKind::COUNTUP(_ident, _start_expr, _end_expr, _body) => {}
            res::StatementNodeKind::ASM(_asm_literals) => {}
        }
    }

    fn check_expression(
        &mut self,
        ex: &res::ExpressionNode,
        all_funcs: &[res::PFunction],
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

                if let None = locals.get(&defined_name) {
                    let ident_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::used_undefined_auto_var(
                        defined_name.clone(),
                        ident_pos,
                    ));

                    return None;
                }

                let local_var = locals.get(&defined_name).unwrap();
                Some(self.get_global_type_from(local_var.get_type()))
            }

            res::ExpressionNodeKind::CALL(name, _args) => {
                let called_name = res::IdentName::last_name(name);
                let search_result =
                    all_funcs.binary_search_by(|f| f.copy_func_name().cmp(&called_name));

                if search_result.is_err() {
                    let call_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::called_undefined_function(
                        called_name,
                        call_pos,
                    ));

                    return None;
                }

                let return_type = all_funcs[search_result.unwrap()].get_return_type();
                Some(self.get_global_type_from(return_type))
            }

            _ => None,
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
