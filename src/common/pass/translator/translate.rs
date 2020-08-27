use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use id_arena::Arena;

use crate::common::analyze_resource::peachili_type::Type;
use crate::common::option;
use crate::common::{ast, peachili_type, three_address_code as tac};

type ValueCache = BTreeMap<ast::ExpressionNode, tac::ValueId>;

/// 4つ組生成のメインルーチン
pub fn translate_ir(
    fn_arena: ast::FnArena,
    ast_root: ast::ASTRoot,
    type_env: &BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
    target: option::Target,
) -> tac::IRModule {
    let mut ir_module: tac::IRModule = Default::default();

    // 関数列をイテレートし，IRFunctionの列に変換する
    for fn_id in ast_root.funcs.iter() {
        if let Ok(fn_arena) = fn_arena.lock() {
            if let Some(ast_fn) = fn_arena.get(*fn_id) {
                // 呼び出されていない関数はコンパイル対象としない
                if ast_fn.name != "main" && !ast_root.called_functions.contains(&ast_fn.full_path()) {
                    continue;
                }

                let ir_fn = gen_ir_fn(ast_fn, type_env, target);
                let ir_fn_id = ir_module.fn_allocator.alloc(ir_fn);
                ir_module.funcs.push(ir_fn_id);
            }
        }
    }

    ir_module
}

/// 関数単位でIRに変換する
fn gen_ir_fn(
    ast_fn: &ast::Function,
    type_env: &BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
    target: option::Target,
) -> tac::IRFunction {
    // コード生成に必要な情報が多いので，構造体にまとめてメンバでやり取りする
    let mut function_translator = FunctionTranslator::new(
        ast_fn.expr_arena.clone(),
        ast_fn.stmt_arena.clone(),
        type_env,
        ast_fn.name.clone(),
        target,
    );

    // Statement をループして，それぞれをIRに変換する
    for stmt_id in ast_fn.stmts.iter() {
        function_translator.gen_ir_from_stmt(&stmt_id);
    }

    // IRFunctionの生成
    tac::IRFunction {
        name: ast_fn.full_path(),
        code_allocator: Arc::new(Mutex::new(function_translator.code_arena)),
        value_allocator: Arc::new(Mutex::new(function_translator.value_arena)),
        codes: function_translator.codes,
        fn_ty: type_env
            .get(&ast_fn.full_path())
            .unwrap()
            .get(&ast_fn.full_path())
            .unwrap()
            .clone(),
        args: ast_fn
            .get_parameters()
            .iter()
            .map(|(name, _)| name.to_string())
            .collect(),
    }
}

/// IR生成に必要な情報をまとめあげた構造体
struct FunctionTranslator<'a> {
    /// IRの最小単位のアロケータ
    code_arena: Arena<tac::Code>,
    /// IRで用いられるValue構造体のアロケータ
    /// (複数ヶ所で同じValueが参照されるために，Idでラップして取り回す)
    value_arena: Arena<tac::Value>,
    /// Temp変数用のシーケンス番号
    temp_number: usize,
    /// ラベル用のシーケンス番号
    label_number: usize,
    /// DAG生成と同じようにするため，
    /// 同じ式からは同じIdを返す
    value_cache: ValueCache,
    /// IR列
    codes: Vec<tac::CodeId>,
    fn_name: String,
    expr_arena: ast::ExprArena,
    stmt_arena: ast::StmtArena,
    type_env: &'a BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
    target: option::Target,
}

impl<'a> FunctionTranslator<'a> {
    /// 文単位でIRに変換する
    #[allow(clippy::single_match)]
    fn gen_ir_from_stmt(&mut self, stmt_id: &ast::StNodeId) -> Option<tac::ValueId> {
        let stmt = self
            .stmt_arena
            .lock()
            .unwrap()
            .get(*stmt_id)
            .unwrap()
            .clone();
        match stmt.get_kind() {
            // return v1;
            ast::StatementNodeKind::RETURN { expr: expr_id } => self.gen_from_return_stmt(expr_id),
            ast::StatementNodeKind::VARINIT {
                ident_name,
                type_name: _,
                expr,
            } => self.gen_from_varinit_stmt(ident_name.clone(), expr),
            ast::StatementNodeKind::ASM { stmts } => {
                for stmt_id in stmts.iter() {
                    let asm_str = self.gen_ir_from_stmt(stmt_id);
                    self.add_code_with_allocation(tac::CodeKind::ASM {
                        value: asm_str.unwrap(),
                    });
                }

                None
            }
            ast::StatementNodeKind::CONST {
                ident_name,
                type_name: _,
                expr,
            } => self.gen_from_varinit_stmt(ident_name.clone(), expr),
            ast::StatementNodeKind::IFRET { expr: expr_id } => {
                let ex_v = self.gen_ir_from_expr(expr_id);
                Some(ex_v)
            }
            ast::StatementNodeKind::EXPR { expr: expr_id } => Some(self.gen_ir_from_expr(expr_id)),
            ast::StatementNodeKind::DECLARE {
                ident_name: _,
                type_name: _,
            } => None,
            _ => unimplemented!(),
        }
    }

    /// return-statement の変換
    fn gen_from_return_stmt(&mut self, expr_id: &ast::ExNodeId) -> Option<tac::ValueId> {
        // compile expression, return value.
        let expr_id = self.gen_ir_from_expr(expr_id);
        self.add_code_with_allocation(tac::CodeKind::RETURN { value: expr_id });

        None
    }

    /// varinit-statement の変換
    fn gen_from_varinit_stmt(
        &mut self,
        id_name: String,
        expr_id: &ast::ExNodeId,
    ) -> Option<tac::ValueId> {
        let id_type = self.copy_type_in_cur_func(&id_name);
        let id_value = self.value_arena.alloc(tac::Value {
            kind: tac::ValueKind::ID { name: id_name },
            ty: id_type,
        });
        let expr_id = self.gen_ir_from_expr(expr_id);
        self.add_code_with_allocation(tac::CodeKind::ASSIGN {
            value: expr_id,
            result: id_value,
        });

        None
    }

    fn gen_lvalue(&mut self, expr_id: &ast::ExNodeId) -> tac::ValueId {
        let expr = self.copy_ast_expr(expr_id);

        match expr.get_kind() {
            ast::ExpressionNodeKind::IDENTIFIER { names: _ } => {
                self.gen_ir_from_unop_expr("&", &expr, expr_id)
            }
            ast::ExpressionNodeKind::DEREFERENCE { value } => {
                let inner_v = self.gen_lvalue(value);
                let result_v = self.gen_result_temp(
                    self.value_arena
                        .get(inner_v)
                        .unwrap()
                        .ty
                        .pointer_to()
                        .clone(),
                );
                self.add_code_with_allocation(tac::CodeKind::DEREFERENCE {
                    value: inner_v,
                    result: result_v,
                });
                result_v
            }
            ast::ExpressionNodeKind::MEMBER { id, member } => {
                // 構造体のベースアドレスをレジスタにロード
                let id_v = self.gen_lvalue(id);
                // メンバオフセットをプラスする
                let id_names = self.copy_ast_expr(id).copy_names();
                let id_type = self.copy_type_in_cur_func(&id_names.join("::"));
                let member_type = id_type.get_members().get(member).unwrap();

                let member_addr =
                    self.gen_result_temp(Type::new_pointer(*member_type.0.clone(), self.target));
                let member_offset_id = self
                    .value_arena
                    .alloc(tac::Value::new_int64(member_type.1 as i64, self.target));
                self.add_code_with_allocation(tac::CodeKind::SUB {
                    lop: id_v,
                    rop: member_offset_id,
                    result: member_addr,
                });

                member_addr
            }
            _ => unreachable!(),
        }
    }

    /// 式単位でIRに変換する
    fn gen_ir_from_expr(&mut self, expr_id: &ast::ExNodeId) -> tac::ValueId {
        let expr = self.copy_ast_expr(expr_id);
        match expr.get_kind() {
            // PRIMARY
            // これらはcacheしなくて良い
            ast::ExpressionNodeKind::INTEGER { value } => self
                .value_arena
                .alloc(tac::Value::new_int64(*value, self.target)),
            ast::ExpressionNodeKind::UINTEGER { value } => self
                .value_arena
                .alloc(tac::Value::new_uint64(*value, self.target)),
            ast::ExpressionNodeKind::IDENTIFIER { names } => {
                self.value_arena.alloc(tac::Value::new(
                    tac::ValueKind::ID {
                        name: names.join("::"),
                    },
                    self.copy_type_in_cur_func(&names.join("::")),
                ))
            }
            ast::ExpressionNodeKind::BOOLEAN { truth } => self
                .value_arena
                .alloc(tac::Value::new_boolean(*truth, self.target)),
            ast::ExpressionNodeKind::STRING { contents } => self.value_arena.alloc(
                tac::Value::new_string_literal(contents.to_string(), self.target),
            ),
            ast::ExpressionNodeKind::MEMBER { id, member } => {
                // 両方のオペランドをIRに変換する
                let id = self.gen_ir_from_expr(id);

                // 計算結果をTEMP変数に格納するコードを生成
                let st_type = self.value_arena.get(id).unwrap().ty.clone();
                let member_type = st_type.get_members().get(member).unwrap().0.clone();
                let result_v = self.gen_result_temp(*member_type);

                self.add_code_with_allocation(tac::CodeKind::MEMBER {
                    id,
                    member: member.to_string(),
                    result: result_v,
                });

                result_v
            }

            // 代入式
            ast::ExpressionNodeKind::ASSIGN { lhs, rhs } => {
                // オペランドをIRに変換する
                let ident_id = self.gen_lvalue(lhs);
                let value_id = self.gen_ir_from_expr(rhs);

                self.add_code_with_allocation(tac::CodeKind::STORE {
                    value: value_id,
                    result: ident_id,
                });
                value_id
            }

            // 単項演算
            // 計算結果を格納するTMP変数を返す
            ast::ExpressionNodeKind::NEG { value } => self.gen_ir_from_unop_expr("-", &expr, value),
            ast::ExpressionNodeKind::ADDRESSOF { value } => {
                self.gen_ir_from_unop_expr("&", &expr, value)
            }
            ast::ExpressionNodeKind::DEREFERENCE { value } => {
                self.gen_ir_from_unop_expr("*", &expr, value)
            }

            // 二項演算
            // 計算結果を格納するTMP変数を返す
            ast::ExpressionNodeKind::ADD { lhs, rhs } => {
                self.gen_ir_from_binop_expr("+", &expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::SUB { lhs, rhs } => {
                self.gen_ir_from_binop_expr("-", &expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::MUL { lhs, rhs } => {
                self.gen_ir_from_binop_expr("*", &expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::DIV { lhs, rhs } => {
                self.gen_ir_from_binop_expr("/", &expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::CALL { names, args } => {
                self.gen_ir_from_call_expr(names.join("::"), args)
            }
            ast::ExpressionNodeKind::IF {
                cond_ex,
                body,
                alter,
            } => self.gen_ir_from_if_expr(cond_ex, body, alter),
        }
    }

    /// 呼び出し式のIRを生成する
    fn gen_ir_from_call_expr(&mut self, name: String, args: &[ast::ExNodeId]) -> tac::ValueId {
        // 引数を順にIRに変換 -> param {value} を生成
        self.gen_parameters(args);

        // 計算結果をTEMP変数に格納するコードを生成
        let call_fn_type = self.copy_type_in_called_func(&name, &name);
        let result_v = self.gen_result_temp(call_fn_type.clone());

        // call funcの生成
        let call_kind = tac::CodeKind::CALL {
            name: self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::ID { name },
                ty: call_fn_type,
            }),
            result: result_v,
        };
        self.add_code_with_allocation(call_kind);

        result_v
    }

    /// 各パラメータをコンパイルする
    fn gen_parameters(&mut self, args: &[ast::ExNodeId]) {
        for arg_id in args.iter() {
            let arg_value_id = self.gen_ir_from_expr(arg_id);
            self.add_code_with_allocation(tac::CodeKind::PARAM {
                value: arg_value_id,
            });
        }
    }

    /// 演算のIRを生成する
    fn gen_ir_from_unop_expr(
        &mut self,
        operator: &str,
        expr: &ast::ExpressionNode,
        value_id: &ast::ExNodeId,
    ) -> tac::ValueId {
        // 既に変換済みの式である場合，キャッシュ済みであるtemp valueを得る
        if let Some(tmp_v) = self.value_cache.get(expr) {
            return *tmp_v;
        }

        // オペランドをIRに変換する
        let v_id = self.gen_ir_from_expr(value_id);

        // 計算結果をTEMP変数に格納するコードを生成
        let result_v_ty = match operator {
            "-" => self.value_arena.get(v_id).unwrap().ty.clone(),
            "&" => Type::new_pointer(self.value_arena.get(v_id).unwrap().ty.clone(), self.target),
            "*" => self.value_arena.get(v_id).unwrap().ty.pointer_to().clone(),
            _ => unreachable!(),
        };
        let result_v = self.gen_result_temp(result_v_ty);

        // 生成するIRの種類を決定
        let code_kind = match operator {
            "-" => tac::CodeKind::NEG {
                value: v_id,
                result: result_v,
            },
            "&" => tac::CodeKind::ADDRESSOF {
                value: v_id,
                result: result_v,
            },
            "*" => tac::CodeKind::DEREFERENCE {
                value: v_id,
                result: result_v,
            },
            _ => unreachable!(),
        };

        self.add_code_with_allocation(code_kind);
        self.value_cache.insert(expr.clone(), result_v);

        result_v
    }

    /// 二項演算のIRを生成する
    fn gen_ir_from_binop_expr(
        &mut self,
        operator: &str,
        expr: &ast::ExpressionNode,
        lop_id: &ast::ExNodeId,
        rop_id: &ast::ExNodeId,
    ) -> tac::ValueId {
        // 既に変換済みの式である場合，キャッシュ済みであるtemp valueを得る
        if let Some(tmp_v) = self.value_cache.get(expr) {
            return *tmp_v;
        }

        // 両方のオペランドをIRに変換する
        let lop_value_id = self.gen_ir_from_expr(lop_id);
        let rop_value_id = self.gen_ir_from_expr(rop_id);

        // 計算結果をTEMP変数に格納するコードを生成
        let result_v = self.gen_result_temp(self.value_arena.get(lop_value_id).unwrap().ty.clone());

        // 生成するIRの種類を決定
        let code_kind = match operator {
            "+" => tac::CodeKind::ADD {
                lop: lop_value_id,
                rop: rop_value_id,
                result: result_v,
            },
            "-" => tac::CodeKind::SUB {
                lop: lop_value_id,
                rop: rop_value_id,
                result: result_v,
            },
            "*" => tac::CodeKind::MUL {
                lop: lop_value_id,
                rop: rop_value_id,
                result: result_v,
            },
            "/" => tac::CodeKind::DIV {
                lop: lop_value_id,
                rop: rop_value_id,
                result: result_v,
            },
            _ => unreachable!(),
        };

        self.add_code_with_allocation(code_kind);
        self.value_cache.insert(expr.clone(), result_v);

        result_v
    }

    fn gen_ir_from_if_expr(
        &mut self,
        cond_id: &ast::ExNodeId,
        body: &[ast::StNodeId],
        alter: &Option<Vec<ast::StNodeId>>,
    ) -> tac::ValueId {
        //                  | condition_code
        //                  | jump false_label if cond_false
        //                  ---------------------------------
        // true_label    -> | true_block_code
        //                  | jump next_label
        //                  ---------------------------------
        // (false_label) -> | (false_block_code)
        //                  | jump next_label
        //                  ---------------------------------
        // next_label    -> | next_code
        //
        let ifret_temp = self.gen_result_temp(Type::new_int64(self.target));
        self.add_code_with_allocation(tac::CodeKind::ALLOC { temp: ifret_temp });

        let false_label = self.gen_label_without_increment("FALSE");
        let true_label = self.gen_label_without_increment("TRUE");
        let next_label = self.gen_label("NEXT");

        let cond_v = self.gen_ir_from_expr(cond_id);
        let cond_result_tmp =
            self.gen_result_temp(self.value_arena.get(cond_v).unwrap().ty.clone());
        self.add_code_with_allocation(tac::CodeKind::ASSIGN {
            value: cond_v,
            result: cond_result_tmp,
        });
        self.add_code_with_allocation(tac::CodeKind::JUMPIFFALSE {
            label: false_label.to_string(),
            cond_result: cond_result_tmp,
        });
        self.add_code_with_allocation(tac::CodeKind::JUMP {
            label: true_label.clone(),
        });

        // true_block
        self.add_code_with_allocation(tac::CodeKind::LABEL { name: true_label });
        for st_id in body.iter() {
            let stmt = self.stmt_arena.lock().unwrap().get(*st_id).unwrap().clone();
            let result_v = self.gen_ir_from_stmt(st_id);

            if stmt.is_ifret() {
                self.add_code_with_allocation(tac::CodeKind::ASSIGN {
                    value: result_v.unwrap(),
                    result: ifret_temp,
                });
            }
        }
        self.add_code_with_allocation(tac::CodeKind::JUMP {
            label: next_label.clone(),
        });

        // false_block
        self.add_code_with_allocation(tac::CodeKind::LABEL { name: false_label });
        if let Some(alter) = alter {
            for st_id in alter.iter() {
                let stmt = self.stmt_arena.lock().unwrap().get(*st_id).unwrap().clone();
                let result_v = self.gen_ir_from_stmt(st_id);
                if stmt.is_ifret() {
                    self.add_code_with_allocation(tac::CodeKind::ASSIGN {
                        value: result_v.unwrap(),
                        result: ifret_temp,
                    });
                }
            }
        }
        self.add_code_with_allocation(tac::CodeKind::JUMP {
            label: next_label.clone(),
        });

        // next_block
        self.add_code_with_allocation(tac::CodeKind::LABEL { name: next_label });
        ifret_temp
    }

    /// IR生成用
    fn add_code_with_allocation(&mut self, k: tac::CodeKind) -> tac::CodeId {
        let code_id = self.code_arena.alloc(tac::Code { kind: k });
        self.codes.push(code_id);

        code_id
    }

    /// 計算結果を格納するTEMP変数のalloc
    fn gen_result_temp(&mut self, ty: peachili_type::Type) -> tac::ValueId {
        let result_v = self
            .value_arena
            .alloc(tac::Value::new_temp(self.temp_number, ty));
        self.temp_number += 1;
        result_v
    }

    fn gen_label(&mut self, prefix: &str) -> String {
        let l = format!("{}_{}", prefix, self.label_number);
        self.label_number += 1;

        l
    }
    fn gen_label_without_increment(&self, prefix: &str) -> String {
        format!("{}_{}", prefix, self.label_number)
    }

    fn copy_type_in_cur_func(&self, id_name: &str) -> Type {
        self.type_env
            .get(&self.fn_name)
            .unwrap()
            .get(id_name)
            .unwrap()
            .clone()
    }
    fn copy_type_in_called_func(&self, called_fn: &str, id_name: &str) -> Type {
        self.type_env
            .get(called_fn)
            .unwrap()
            .get(id_name)
            .unwrap()
            .clone()
    }
    fn copy_ast_expr(&self, expr_id: &ast::ExNodeId) -> ast::ExpressionNode {
        self.expr_arena
            .lock()
            .unwrap()
            .get(*expr_id)
            .unwrap()
            .clone()
    }

    fn new(
        expr_arena: ast::ExprArena,
        stmt_arena: ast::StmtArena,
        type_env: &'a BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
        fn_name: String,
        target: option::Target,
    ) -> Self {
        Self {
            code_arena: Arena::new(),
            value_arena: Arena::new(),
            temp_number: 0,
            label_number: 0,
            value_cache: Default::default(),
            codes: Vec::new(),
            expr_arena,
            stmt_arena,
            type_env,
            fn_name,
            target,
        }
    }
}

#[cfg(test)]
mod translate_tests {}
