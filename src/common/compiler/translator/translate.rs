use crate::common::{ast, peachili_type, three_address_code as tac};
use id_arena::Arena;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

type ValueCache = BTreeMap<ast::ExpressionNode, tac::ValueId>;

/// 4つ組生成のメインルーチン
pub fn translate_ir(
    fn_arena: ast::FnArena,
    ast_root: ast::ASTRoot,
    type_env: &BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
) -> tac::IRModule {
    let mut ir_module: tac::IRModule = Default::default();

    // 関数列をイテレートし，IRFunctionの列に変換する
    for fn_id in ast_root.funcs.iter() {
        if let Ok(fn_arena) = fn_arena.lock() {
            if let Some(ast_fn) = fn_arena.get(*fn_id) {
                let ir_fn = gen_ir_fn(ast_fn, type_env);
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
) -> tac::IRFunction {
    // コード生成に必要な情報が多いので，構造体にまとめてメンバでやり取りする
    let mut function_translator =
        FunctionTranslator::new(ast_fn.expr_arena.clone(), ast_fn.stmt_arena.clone());

    // Statement をループして，それぞれをIRに変換する
    for stmt_id in ast_fn.stmts.iter() {
        function_translator.gen_ir_from_stmt(&stmt_id);
    }

    // IRFunctionの生成
    tac::IRFunction {
        name: ast_fn.name.clone(),
        code_allocator: Arc::new(Mutex::new(function_translator.code_arena)),
        value_allocator: Arc::new(Mutex::new(function_translator.value_arena)),
        codes: function_translator.codes,
        return_type: type_env
            .get(&ast_fn.name)
            .unwrap()
            .get(&ast_fn.name)
            .unwrap()
            .clone(),
    }
}

/// IR生成に必要な情報をまとめあげた構造体
struct FunctionTranslator {
    /// IRの最小単位のアロケータ
    code_arena: Arena<tac::Code>,
    /// IRで用いられるValue構造体のアロケータ
    /// (複数ヶ所で同じValueが参照されるために，Idでラップして取り回す)
    value_arena: Arena<tac::Value>,
    /// Temp変数用のシーケンス番号
    temp_number: usize,
    /// DAG生成と同じようにするため，
    /// 同じ式からは同じIdを返す
    value_cache: ValueCache,
    /// IR列
    codes: Vec<tac::CodeId>,
    expr_arena: ast::ExprArena,
    stmt_arena: ast::StmtArena,
}

impl FunctionTranslator {
    /// 文単位でIRに変換する
    #[allow(clippy::single_match)]
    fn gen_ir_from_stmt(&mut self, stmt_id: &ast::StNodeId) {
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
                ident_name, type_name: _, expr
            } => self.gen_from_varinit_stmt(ident_name.clone(), expr),
            ast::StatementNodeKind::ASM { stmts } => {
                for stmt_id in stmts.iter() {
                    self.gen_ir_from_stmt(stmt_id);
                }
            }
            ast::StatementNodeKind::EXPR { expr: expr_id } => {
                let expr = self
                    .expr_arena
                    .lock()
                    .unwrap()
                    .get(*expr_id)
                    .unwrap()
                    .clone();
                self.gen_ir_from_expr(&expr);
            }
            _ => unimplemented!()
        }
    }

    /// return-statement の変換
    fn gen_from_return_stmt(&mut self, expr_id: &ast::ExNodeId) {
        let expr = self
            .expr_arena
            .lock()
            .unwrap()
            .get(*expr_id)
            .unwrap()
            .clone();
        let expr_id = self.gen_ir_from_expr(&expr);
        let return_code_id = self.code_arena.alloc(tac::Code {
            kind: tac::CodeKind::RETURN { value: expr_id },
        });
        self.codes.push(return_code_id);
    }

    /// varinit-statement の変換
    fn gen_from_varinit_stmt(&mut self, id_name: String, expr_id: &ast::ExNodeId) {
        let id_value = self.value_arena.alloc(tac::Value {
            kind: tac::ValueKind::ID { name: id_name }
        });
        let expr = self
            .expr_arena
            .lock()
            .unwrap()
            .get(*expr_id)
            .unwrap()
            .clone();
        let expr_id = self.gen_ir_from_expr(&expr);
        let assign_code_id = self.code_arena.alloc(tac::Code {
            kind: tac::CodeKind::ASSIGN { value: expr_id, result: id_value },
        });
        self.codes.push(assign_code_id);
    }

    // 式単位でIRに変換する
    fn gen_ir_from_expr(&mut self, expr: &ast::ExpressionNode) -> tac::ValueId {
        match expr.get_kind() {
            // PRIMARY
            // これらはcacheしなくて良い
            ast::ExpressionNodeKind::INTEGER { value } => self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::INTLITERAL { value: *value },
            }),
            ast::ExpressionNodeKind::UINTEGER { value } => self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::UINTLITERAL { value: *value },
            }),
            ast::ExpressionNodeKind::IDENTIFIER { names } => self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::ID {
                    name: names.join("::"),
                },
            }),
            ast::ExpressionNodeKind::BOOLEAN { truth } => self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::BOOLEANLITERAL { truth: *truth },
            }),
            ast::ExpressionNodeKind::STRING { contents } => self.value_arena.alloc(tac::Value {
                kind: tac::ValueKind::STRINGLITERAL { contents: contents.clone() },
            }),

            // 代入式
            ast::ExpressionNodeKind::ASSIGN { lhs, rhs } => {
                // オペランドをIRに変換する
                let ident_node = self.expr_arena.lock().unwrap().get(*lhs).unwrap().clone();
                let ident_id = self.gen_ir_from_expr(&ident_node);
                let value_node = self.expr_arena.lock().unwrap().get(*rhs).unwrap().clone();
                let value_id = self.gen_ir_from_expr(&value_node);

                // 生成結果を変数に格納するコードを生成
                let code_id = self.code_arena.alloc(tac::Code {
                    kind: tac::CodeKind::ASSIGN {
                        value: value_id,
                        result: ident_id,
                    },
                });
                self.codes.push(code_id);
                value_id
            }

            // 単項演算
            // 計算結果を格納するTMP変数を返す
            ast::ExpressionNodeKind::NEG { value } => self.gen_ir_from_unop_expr("-", expr, value),
            ast::ExpressionNodeKind::ADDRESSOF { value } => {
                self.gen_ir_from_unop_expr("&", expr, value)
            }
            ast::ExpressionNodeKind::DEREFERENCE { value } => {
                self.gen_ir_from_unop_expr("*", expr, value)
            }

            // 二項演算
            // 計算結果を格納するTMP変数を返す
            ast::ExpressionNodeKind::ADD { lhs, rhs } => {
                self.gen_ir_from_binop_expr("+", expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::SUB { lhs, rhs } => {
                self.gen_ir_from_binop_expr("-", expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::MUL { lhs, rhs } => {
                self.gen_ir_from_binop_expr("*", expr, lhs, rhs)
            }
            ast::ExpressionNodeKind::DIV { lhs, rhs } => {
                self.gen_ir_from_binop_expr("/", expr, lhs, rhs)
            }
            // メンバアクセス式も二項演算のようにしておく
            ast::ExpressionNodeKind::MEMBER { id, member } => {
                self.gen_ir_from_binop_expr(".", expr, id, member)
            }
            ast::ExpressionNodeKind::CALL { names, args } => {
                self.gen_ir_from_call_expr(names.join("::"), args)
            }
            _ => unimplemented!()
        }
    }

    /// 呼び出し式のIRを生成する
    fn gen_ir_from_call_expr(&mut self, name: String, args: &[ast::ExNodeId]) -> tac::ValueId {
        // 引数を順にIRに変換 -> param {value} を生成
        for arg_id in args.iter() {
            let arg = self.expr_arena.lock().unwrap().get(*arg_id).unwrap().clone();
            let arg_value_id = self.gen_ir_from_expr(&arg);
            let param_id = self.code_arena.alloc(tac::Code {
                kind: tac::CodeKind::PARAM {
                    value: arg_value_id,
                },
            });
            self.codes.push(param_id);
        }

        // 計算結果をTEMP変数に格納するコードを生成
        let result_v = self.value_arena.alloc(tac::Value {
            kind: tac::ValueKind::TEMP {
                number: self.temp_number,
            },
        });
        self.temp_number += 1;

        let call_id = self.code_arena.alloc(tac::Code {
            kind: tac::CodeKind::CALL {
                name: self.value_arena.alloc(tac::Value {
                    kind: tac::ValueKind::ID {
                        name
                    }
                }),
                result: result_v,
            }
        });
        self.codes.push(call_id);

        result_v
    }

    /// 演算のIRを生成する
    fn gen_ir_from_unop_expr(
        &mut self,
        operator: &str,
        expr: &ast::ExpressionNode,
        value: &ast::ExNodeId,
    ) -> tac::ValueId {
        // 既に変換済みの式である場合，キャッシュ済みであるtemp valueを得る
        if let Some(tmp_v) = self.value_cache.get(expr) {
            return *tmp_v;
        }

        // オペランドをIRに変換する
        let v_node = self.expr_arena.lock().unwrap().get(*value).unwrap().clone();
        let v_id = self.gen_ir_from_expr(&v_node);

        // 計算結果をTEMP変数に格納するコードを生成
        let result_v = self.value_arena.alloc(tac::Value {
            kind: tac::ValueKind::TEMP {
                number: self.temp_number,
            },
        });

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
        let code = tac::Code { kind: code_kind };
        let code_id = self.code_arena.alloc(code);
        self.temp_number += 1;
        self.value_cache.insert(expr.clone(), result_v);
        self.codes.push(code_id);

        result_v
    }

    /// 二項演算のIRを生成する
    fn gen_ir_from_binop_expr(
        &mut self,
        operator: &str,
        expr: &ast::ExpressionNode,
        lop: &ast::ExNodeId,
        rop: &ast::ExNodeId,
    ) -> tac::ValueId {
        // 既に変換済みの式である場合，キャッシュ済みであるtemp valueを得る
        if let Some(tmp_v) = self.value_cache.get(expr) {
            return *tmp_v;
        }

        // 両方のオペランドをIRに変換する
        let lop_value = self.expr_arena.lock().unwrap().get(*lop).unwrap().clone();
        let lop_value_id = self.gen_ir_from_expr(&lop_value);
        let rop_value = self.expr_arena.lock().unwrap().get(*rop).unwrap().clone();
        let rop_value_id = self.gen_ir_from_expr(&rop_value);

        // 計算結果をTEMP変数に格納するコードを生成
        let result_v = self.value_arena.alloc(tac::Value {
            kind: tac::ValueKind::TEMP {
                number: self.temp_number,
            },
        });

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
            "." => tac::CodeKind::MEMBER {
                id: lop_value_id,
                member: rop_value_id,
                result: result_v,
            },
            _ => unreachable!(),
        };
        let code = tac::Code { kind: code_kind };
        let code_id = self.code_arena.alloc(code);
        self.temp_number += 1;
        self.value_cache.insert(expr.clone(), result_v);
        self.codes.push(code_id);

        result_v
    }

    fn new(expr_arena: ast::ExprArena, stmt_arena: ast::StmtArena) -> Self {
        Self {
            code_arena: Arena::new(),
            value_arena: Arena::new(),
            temp_number: 0,
            value_cache: Default::default(),
            codes: Vec::new(),
            expr_arena,
            stmt_arena,
        }
    }
}

#[cfg(test)]
mod translate_tests {
    use super::*;
    use crate::common::token;

    #[test]
    fn gen_ir_from_stmt_test() {
        let expr_arena: ast::ExprArena = Arc::new(Mutex::new(Arena::new()));
        let stmt_arena: ast::StmtArena = Arc::new(Mutex::new(Arena::new()));
        let mut translator = FunctionTranslator::new(expr_arena, stmt_arena);

        let return_stmt = new_simple_return(translator.expr_arena.clone());

        translator.gen_ir_from_stmt(&return_stmt);

// v0 <- 1 + 2
// v1 <- v0 + 3
// return v1
        assert_eq!(3, translator.codes.len());
        assert_eq!(2, translator.temp_number);
        assert_eq!(2, translator.value_cache.len());
    }

    #[test]
    fn gen_ir_from_expr_test() {
        let expr_arena: ast::ExprArena = Arc::new(Mutex::new(Arena::new()));
        let stmt_arena: ast::StmtArena = Arc::new(Mutex::new(Arena::new()));
        let mut translator = FunctionTranslator::new(expr_arena, stmt_arena);

        gen_ir_from_integer_literal_test(&mut translator);
        gen_ir_from_add_node_test(&mut translator);
    }

    fn gen_ir_from_integer_literal_test(translator: &mut FunctionTranslator) {
        let int_node = ast::ExpressionNode::new_integer(30, Default::default());
        let int_v = translator.gen_ir_from_expr(&int_node);
        assert_eq!(
            tac::ValueKind::INTLITERAL { value: 30 },
            translator.value_arena.get(int_v).unwrap().kind
        );
    }

    fn gen_ir_from_add_node_test(translator: &mut FunctionTranslator) {
        let add_node = new_simple_add(translator.expr_arena.clone());
        let add_v = translator.gen_ir_from_expr(&add_node);

// v0 <- 1 + 2
// v1 <- v0 + 3
        assert_eq!(2, translator.codes.len());
        assert_eq!(2, translator.temp_number);
        assert_eq!(2, translator.value_cache.len());

        let add_result = translator.value_arena.get(add_v);
        assert!(add_result.is_some());
        let add_result = add_result.unwrap();
        assert_eq!(tac::ValueKind::TEMP { number: 1 }, add_result.kind);
    }

    fn new_simple_add(expr_arena: ast::ExprArena) -> ast::ExpressionNode {
// 1 + 2 + 3
        let one_id = expr_arena
            .lock()
            .unwrap()
            .alloc(ast::ExpressionNode::new_integer(1, Default::default()));
        let two_id = expr_arena
            .lock()
            .unwrap()
            .alloc(ast::ExpressionNode::new_integer(2, Default::default()));
        let three_id = expr_arena
            .lock()
            .unwrap()
            .alloc(ast::ExpressionNode::new_integer(3, Default::default()));
        let add_id = expr_arena
            .lock()
            .unwrap()
            .alloc(ast::ExpressionNode::new_binop(
                &token::TokenKind::PLUS,
                one_id,
                two_id,
                Default::default(),
            ));
        ast::ExpressionNode::new_binop(
            &token::TokenKind::PLUS,
            add_id,
            three_id,
            Default::default(),
        )
    }

    fn new_simple_return(expr_arena: ast::ExprArena) -> ast::StatementNode {
        let add_node = new_simple_add(expr_arena.clone());
        let expr_id = expr_arena.lock().unwrap().alloc(add_node);

        ast::StatementNode::new(
            ast::StatementNodeKind::RETURN { expr: expr_id },
            Default::default(),
        )
    }

    #[allow(dead_code)]
    fn new_simple_fn() -> ast::Function {
//
// func sample() Noreturn {
//   return 1 + 2 + 3;
// }

        let stmt_arena: ast::StmtArena = Arc::new(Mutex::new(Arena::new()));
        let expr_arena: ast::ExprArena = Arc::new(Mutex::new(Arena::new()));

        let return_id = stmt_arena
            .lock()
            .unwrap()
            .alloc(new_simple_return(expr_arena.clone()));

        ast::Function {
            name: "sample".to_string(),
            stmts: vec![return_id],
            return_type: "Noreturn".to_string(),
            args: Default::default(),
            pos: Default::default(),
            module_name: "translate_tests".to_string(),
            stmt_arena,
            expr_arena: Arc::new(Mutex::new(Arena::new())),
        }
    }
}
