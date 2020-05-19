use std::collections::BTreeMap;
use std::time;

use crate::common::{error as er, operate, option};
use crate::compiler::resource as res;

// 型チェック時に毎回タイプを生成するとコストがかかる
// ここではグローバルな実体の参照を取り回すことで，型検査を実装する
pub fn type_check_phase(
    build_option: &option::BuildOption,
    root: &res::ASTRoot,
    tld_map: &BTreeMap<String, res::TopLevelDecl>,
) {
    let function_number = root.get_functions().len() as u64;
    let type_check_pb = indicatif::ProgressBar::new(function_number);
    type_check_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    let func_map = root.get_functions();
    for (func_name, func) in func_map.iter() {
        type_check_pb.set_message(&format!("type check in {}", func_name));

        let errors = type_check_fn(build_option, tld_map, func);

        if !errors.is_empty() {
            let module_path = func.copy_module_path();
            operate::emit_all_errors_and_exit(&errors, &module_path, build_option);
        }

        type_check_pb.inc(1);
    }
    let end = time::Instant::now();

    type_check_pb.finish_with_message(&format!("type check done!(in {:?})", end - start));
}

fn type_check_fn(
    build_opt: &option::BuildOption,
    tld_map: &BTreeMap<String, res::TopLevelDecl>,
    this_func: &res::PFunction,
) -> Vec<er::CompileError> {
    let stmts = this_func.get_statements();
    let locals = this_func.get_locals();

    let mut checker = res::TypeChecker::new(build_opt);
    for st in stmts.iter() {
        checker.check_statement(st, tld_map, locals);
    }

    checker.give_errors()
}

impl<'a> res::TypeChecker<'a> {
    fn check_statement(
        &mut self,
        st: &res::StatementNode,
        tld_map: &BTreeMap<String, res::TopLevelDecl>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        match &st.kind {
            res::StatementNodeKind::RETURN(_return_expr) => None,
            res::StatementNodeKind::IFRET(return_expr) => {
                self.check_expression(return_expr, tld_map, locals)
            }
            res::StatementNodeKind::EXPR(expr) => self.check_expression(expr, tld_map, locals),
            res::StatementNodeKind::VARDECL => None,
            res::StatementNodeKind::COUNTUP(_ident, _start_expr, _end_expr, _body) => None,
            res::StatementNodeKind::ASM(_asm_literals) => None,
        }
    }

    fn check_expression(
        &mut self,
        ex: &res::ExpressionNode,
        tld_map: &BTreeMap<String, res::TopLevelDecl>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        match &ex.kind {
            res::ExpressionNodeKind::INTEGER(_v) => Some(res::PType::GLOBAL_INT_TYPE),
            res::ExpressionNodeKind::UNSIGNEDINTEGER(_v) => Some(res::PType::GLOBAL_UINT_TYPE),
            res::ExpressionNodeKind::STRLIT(_contents, _hash) => Some(res::PType::GLOBAL_STR_TYPE),
            res::ExpressionNodeKind::TRUE | res::ExpressionNodeKind::FALSE => {
                Some(res::PType::GLOBAL_BOOLEAN_TYPE)
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
                let var_type = res::PType::get_global_type_from(local_var.get_type());

                // 型mapからも探す
                if let res::PTypeKind::UNRESOLVED(type_name) = &var_type.kind {
                    eprintln!("type_name -> {}", type_name);

                    if let Some(src_type) = tld_map.get(type_name) {
                        let alias_type = res::PType::get_global_type_from(src_type.get_src_type());
                        return Some(alias_type);
                    }
                }

                Some(var_type)
            }

            res::ExpressionNodeKind::CALL(name, _args) => {
                let called_name = res::IdentName::last_name(name);

                let called_decl_opt = tld_map.get(&called_name);

                if called_decl_opt.is_none() {
                    let call_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::called_undefined_function(
                        called_name,
                        call_pos,
                    ));

                    return None;
                }

                let called_decl = called_decl_opt.unwrap();

                Some(res::PType::get_global_type_from(
                    called_decl.get_return_type(),
                ))
            }

            res::ExpressionNodeKind::ADD(lop, rop)
            | res::ExpressionNodeKind::SUB(lop, rop)
            | res::ExpressionNodeKind::MUL(lop, rop)
            | res::ExpressionNodeKind::DIV(lop, rop) => {
                let lop_type = self.try_to_resolve_expression(lop, tld_map, locals);
                let rop_type = self.try_to_resolve_expression(rop, tld_map, locals);

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

                lop_type
            }

            res::ExpressionNodeKind::ASSIGN(lvalue, rvalue) => {
                let lvalue_type = self.try_to_resolve_expression(lvalue, tld_map, locals);
                let rvalue_type = self.try_to_resolve_expression(rvalue, tld_map, locals);

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
                if self.detect_conditional_expression_error(cond_expr, tld_map, locals) {
                    return None;
                }
                self.check_block_statement(body, tld_map, locals)
            }
            res::ExpressionNodeKind::IFELSE(cond_expr, body, alter) => {
                if self.detect_conditional_expression_error(cond_expr, tld_map, locals) {
                    return None;
                }

                let body_type = self.check_block_statement(body, tld_map, locals);
                let alter_type = self.check_block_statement(alter, tld_map, locals);

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
        tld_map: &BTreeMap<String, res::TopLevelDecl>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> bool {
        // 型が解決できるかチェック
        let cond_expr_type = self.try_to_resolve_expression(cond_expr, tld_map, locals);
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
        tld_map: &BTreeMap<String, res::TopLevelDecl>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        for st in block.iter() {
            if !self.is_ifret(st) {
                continue;
            }

            // ifret-statementのチェック
            return self.check_statement(st, tld_map, locals);
        }

        None
    }

    fn try_to_resolve_expression(
        &mut self,
        ex: &res::ExpressionNode,
        tld_map: &BTreeMap<String, res::TopLevelDecl>,
        locals: &BTreeMap<String, res::PVariable>,
    ) -> Option<res::PType> {
        let ex_type = self.check_expression(ex, tld_map, locals);

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
}
