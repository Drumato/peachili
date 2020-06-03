use std::collections::BTreeMap;
use std::time;

use crate::common::{error as er, module, operate, option, position as pos};
use crate::compiler::general::resource as res;

// 型チェック時に毎回タイプを生成するとコストがかかる
// ここではグローバルな実体の参照を取り回すことで，型検査を実装する
pub fn type_check_phase(
    build_option: &option::BuildOption,
    root: &res::ASTRoot,
    tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
    module_allocator: &module::ModuleAllocator,
) {
    // プログレスバーの初期化
    let function_number = root.get_functions().len() as u64;
    let type_check_pb = indicatif::ProgressBar::new(function_number);
    type_check_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    let func_map = root.get_functions();
    let mut main_exists = false;

    for (func_name_id, func) in func_map.iter() {
        let const_pool = module_allocator
            .get_module_ref(&func.module_id)
            .unwrap()
            .get_const_pool_ref();
        let func_name = const_pool.get(*func_name_id).unwrap();

        type_check_pb.set_message(&format!("type check in {}", func_name));

        // mainシンボルの存在をチェック
        if func_name.compare_str("main".to_string()) {
            main_exists = true;

            // mainシンボルの型シグネチャが不正である
            if !func.arg_empty() || func.get_return_type().kind != res::PTypeKind::NORETURN {
                er::CompileError::main_must_have_no_args_and_noreturn()
                    .emit_stderr("", build_option);
                std::process::exit(1);
            }
        }

        let errors = type_check_fn(build_option, tld_map, *func_name_id, func, &const_pool);

        // ある関数をチェックした結果エラーを発見した場合
        if !errors.is_empty() {
            let module_path = func.copy_module_path();
            operate::emit_all_errors_and_exit(&errors, &module_path, build_option);
        }

        type_check_pb.inc(1);
    }

    let end = time::Instant::now();

    // mainシンボルが存在しなかったとき
    if !main_exists {
        er::CompileError::main_must_exist().emit_stderr("", build_option);
        std::process::exit(1);
    }

    type_check_pb.finish_with_message(&format!("type check done!(in {:?})", end - start));
}

fn type_check_fn(
    build_opt: &option::BuildOption,
    tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
    func_name_id: res::PStringId,
    this_func: &res::PFunction,
    const_pool: &res::ConstAllocator,
) -> Vec<er::CompileError> {
    let stmts = this_func.get_statements();
    let locals = this_func.get_locals();

    let mut checker = res::TypeChecker::new(build_opt);
    for st in stmts.iter() {
        checker.check_statement(func_name_id, st, tld_map, locals, const_pool);
    }

    checker.give_errors()
}

impl<'a> res::TypeChecker<'a> {
    fn check_statement(
        &mut self,
        func_name_id: res::PStringId,
        st: &res::StatementNode,
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_pool: &res::ConstAllocator,
    ) -> Option<res::PType> {
        match &st.kind {
            res::StatementNodeKind::RETURN(_return_expr) => {
                let this_func = tld_map.get(&func_name_id).unwrap();

                if this_func.get_return_type().kind == res::PTypeKind::NORETURN {
                    self.detect_error(er::CompileError::return_in_noreturn_function(st.copy_pos()));
                }

                None
            }
            res::StatementNodeKind::IFRET(return_expr) => {
                self.check_expression(func_name_id, return_expr, tld_map, locals, const_pool)
            }
            res::StatementNodeKind::EXPR(expr) => {
                self.check_expression(func_name_id, expr, tld_map, locals, const_pool)
            }
            res::StatementNodeKind::VARDECL => None,
            res::StatementNodeKind::COUNTUP(_ident, _start_expr, _end_expr, _body) => None,
            res::StatementNodeKind::ASM(_asm_literals) => None,
            res::StatementNodeKind::VARINIT(ident, expr) => self.try_to_resolve_assignment(
                func_name_id,
                (st.copy_pos(), ident, expr),
                tld_map,
                locals,
                false,
                const_pool,
            ),
        }
    }

    fn check_expression(
        &mut self,
        func_name_id: res::PStringId,
        ex: &res::ExpressionNode,
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_pool: &res::ConstAllocator,
    ) -> Option<res::PType> {
        match &ex.kind {
            res::ExpressionNodeKind::INTEGER(_v) => Some(res::PType::GLOBAL_INT_TYPE),
            res::ExpressionNodeKind::UNSIGNEDINTEGER(_v) => Some(res::PType::GLOBAL_UINT_TYPE),
            res::ExpressionNodeKind::STRLIT(_contents, _hash) => Some(res::PType::GLOBAL_STR_TYPE),
            res::ExpressionNodeKind::TRUE | res::ExpressionNodeKind::FALSE => {
                Some(res::PType::GLOBAL_BOOLEAN_TYPE)
            }
            res::ExpressionNodeKind::IDENT(name) => {
                let defined_name_id = res::IdentName::last_name(name);

                if locals.get(vec![defined_name_id].as_slice()).is_none() {
                    let ident_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::used_undefined_auto_var(
                        const_pool.get(defined_name_id).unwrap().copy_value(),
                        ident_pos,
                    ));

                    return None;
                }
                let local_var = locals.get(vec![defined_name_id].as_slice()).unwrap();
                let var_type = local_var.get_type().clone();

                // 型mapからも探す
                if let res::PTypeKind::UNRESOLVED(type_name) = &var_type.kind {
                    let type_last = res::IdentName::last_name(type_name);

                    if let Some(src_type) = tld_map.get(&type_last) {
                        return Some(src_type.get_src_type().clone());
                    }
                }

                Some(var_type)
            }

            res::ExpressionNodeKind::CALL(name, args) => {
                // 呼び出されている関数がTLDに存在するかチェック
                let called_name_id = res::IdentName::last_name(name);
                let called_decl_opt = tld_map.get(&called_name_id);

                // TLDに存在しない -> 未定義関数の呼び出し
                if called_decl_opt.is_none() {
                    let call_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::called_undefined_function(
                        const_pool.get(called_name_id).unwrap().copy_value(),
                        call_pos,
                    ));

                    return None;
                }

                let called_decl = called_decl_opt.unwrap();

                let called_fn_args = called_decl.get_args();

                // 引数の数チェック
                if called_fn_args.len() != args.len() {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::arg_number_incorrect(
                        const_pool.get(called_name_id).unwrap().copy_value(),
                        called_fn_args.len(),
                        args.len(),
                        err_pos,
                    ));

                    return None;
                }

                // 引数の型チェック
                for (i, arg) in args.iter().enumerate() {
                    let defined_arg_type = &called_fn_args[i].1;
                    let caller_arg_type = self.try_to_resolve_expression(
                        func_name_id,
                        arg,
                        tld_map,
                        locals,
                        const_pool,
                    );
                    caller_arg_type.as_ref()?;

                    let caller_arg_type = caller_arg_type.unwrap();

                    if defined_arg_type != &caller_arg_type {
                        let err_pos = ex.copy_pos();
                        self.detect_error(er::CompileError::arg_type_incorrect(
                            const_pool.get(called_name_id).unwrap().copy_value(),
                            i,
                            defined_arg_type.clone(),
                            caller_arg_type,
                            err_pos,
                        ));
                        return None;
                    }
                }

                Some(called_decl.get_return_type().clone())
            }

            res::ExpressionNodeKind::NEG(inner_op) => {
                let inner_type = self.try_to_resolve_expression(
                    func_name_id,
                    inner_op,
                    tld_map,
                    locals,
                    const_pool,
                );

                let inner_type = inner_type.unwrap();
                if inner_type.is_unsigned() {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::cannotapplyminusto(inner_type, err_pos));

                    return None;
                }

                Some(inner_type)
            }

            res::ExpressionNodeKind::ADDRESS(inner) => {
                let inner_type = self.try_to_resolve_expression(
                    func_name_id,
                    inner,
                    tld_map,
                    locals,
                    const_pool,
                );

                Some(res::PType::new_pointer(inner_type.unwrap()))
            }

            res::ExpressionNodeKind::DEREF(pointer) => {
                let pointer_type = self.try_to_resolve_expression(
                    func_name_id,
                    pointer,
                    tld_map,
                    locals,
                    const_pool,
                );
                let pointer_type = pointer_type.unwrap();

                if !pointer_type.is_pointer() {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::cannot_dereference_with_not_pointer(
                        pointer_type,
                        err_pos,
                    ));

                    return None;
                }

                Some(pointer_type.dereference())
            }

            res::ExpressionNodeKind::ADD(lop, rop)
            | res::ExpressionNodeKind::SUB(lop, rop)
            | res::ExpressionNodeKind::MUL(lop, rop)
            | res::ExpressionNodeKind::DIV(lop, rop) => {
                let lop_type =
                    self.try_to_resolve_expression(func_name_id, lop, tld_map, locals, const_pool);
                let rop_type =
                    self.try_to_resolve_expression(func_name_id, rop, tld_map, locals, const_pool);

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

            res::ExpressionNodeKind::ASSIGN(lvalue, rvalue) => self.try_to_resolve_assignment(
                func_name_id,
                (ex.copy_pos(), lvalue, rvalue),
                tld_map,
                locals,
                true,
                const_pool,
            ),

            res::ExpressionNodeKind::IF(cond_expr, body) => {
                if self.detect_conditional_expression_error(
                    func_name_id,
                    cond_expr,
                    tld_map,
                    locals,
                    const_pool,
                ) {
                    return None;
                }
                self.check_block_statement(func_name_id, body, tld_map, locals, const_pool)
            }
            res::ExpressionNodeKind::IFELSE(cond_expr, body, alter) => {
                if self.detect_conditional_expression_error(
                    func_name_id,
                    cond_expr,
                    tld_map,
                    locals,
                    const_pool,
                ) {
                    return None;
                }

                let body_type =
                    self.check_block_statement(func_name_id, body, tld_map, locals, const_pool);
                let alter_type =
                    self.check_block_statement(func_name_id, alter, tld_map, locals, const_pool);

                if body_type != alter_type {
                    let err_pos = ex.copy_pos();
                    self.detect_error(er::CompileError::both_blocks_must_be_same_type(err_pos));

                    return None;
                }

                body_type
            }
        }
    }

    /// エラーが見つかれば格納してtrue
    /// そうでなければfalse
    fn detect_conditional_expression_error(
        &mut self,
        func_name_id: res::PStringId,
        cond_expr: &res::ExpressionNode,
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_pool: &res::ConstAllocator,
    ) -> bool {
        // 型が解決できるかチェック
        let cond_expr_type =
            self.try_to_resolve_expression(func_name_id, cond_expr, tld_map, locals, const_pool);
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
        func_name_id: res::PStringId,
        block: &[res::StatementNode],
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_pool: &res::ConstAllocator,
    ) -> Option<res::PType> {
        for st in block.iter() {
            if !self.is_ifret(st) {
                continue;
            }

            // ifret-statementのチェック
            return self.check_statement(func_name_id, st, tld_map, locals, const_pool);
        }

        None
    }

    fn try_to_resolve_expression(
        &mut self,
        func_name_id: res::PStringId,
        ex: &res::ExpressionNode,
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_pool: &res::ConstAllocator,
    ) -> Option<res::PType> {
        let ex_type = self.check_expression(func_name_id, ex, tld_map, locals, const_pool);

        if ex_type.is_none() {
            let err_pos = ex.copy_pos();
            self.detect_error(er::CompileError::unable_to_resolve_expression_type(
                ex.clone(),
                err_pos,
            ));
            return None;
        }
        ex_type
    }

    fn try_to_resolve_assignment(
        &mut self,
        func_name_id: res::PStringId,
        expr_taple: (pos::Position, &res::ExpressionNode, &res::ExpressionNode),
        tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
        locals: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        const_check: bool,
        const_pool: &res::ConstAllocator,
    ) -> Option<res::PType> {
        let assign_pos = expr_taple.0;
        let lvalue = expr_taple.1;
        let rvalue = expr_taple.2;

        let lvalue_type =
            self.try_to_resolve_expression(func_name_id, lvalue, tld_map, locals, const_pool);
        let rvalue_type =
            self.try_to_resolve_expression(func_name_id, rvalue, tld_map, locals, const_pool);

        // try_to_resolve_expression() とは別にエラーを生成
        if rvalue_type.is_none() {
            self.detect_error(er::CompileError::cannot_assignment_unresolved_right_value(
                lvalue.clone(),
                rvalue.clone(),
                assign_pos,
            ));
            return None;
        }

        // 左辺値がconst宣言されているかチェック
        if const_check {
            if let Some(pvar) = locals.get(&lvalue.get_ident_ids()) {
                if pvar.is_constant() {
                    self.detect_error(
                        er::CompileError::cannot_assignment_to_constant_after_initialization(
                            lvalue.clone(),
                            assign_pos,
                        ),
                    );
                    return None;
                }
            }
        }

        if lvalue_type != rvalue_type {
            self.detect_error(
                er::CompileError::both_values_must_be_same_type_in_assignment(
                    lvalue_type.unwrap(),
                    rvalue_type.unwrap(),
                    assign_pos,
                ),
            );
            return None;
        }

        rvalue_type
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
