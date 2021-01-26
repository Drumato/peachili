use crate::compiler::common::frontend::typed_ast;
use crate::compiler::common::frontend::{ast as high_ast, peachili_type};

use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
};

use peachili_type::PeachiliType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConstError {
    #[error("cannot evaluate constant => {name:?}")]
    CannotEvaluate { name: String },
}

/// 基本的には同じ構造になるが，"Typed"にするという点で大きく異なる
pub fn ast_to_lower(
    common_root: &high_ast::ASTRoot,
    resolved_type_env: HashMap<String, PeachiliType>,
) -> Result<typed_ast::Root, Box<dyn std::error::Error>> {
    let mut constants: HashMap<String, typed_ast::Constant> = Default::default();
    let mut lower_functions: Vec<typed_ast::Function> = Vec::with_capacity(common_root.decls.len());

    for common_tld in common_root.decls.iter() {
        match &common_tld.kind {
            high_ast::TopLevelDeclKind::PubConst {
                const_name,
                const_type: _,
                expr,
            } => {
                let const_value = evaluate_constant_expr(const_name, expr)?;
                constants.insert(const_name.to_string(), const_value);
            }
            high_ast::TopLevelDeclKind::Function {
                func_name,
                return_type,
                parameters,
                stmts,
            } => {
                let f = fn_to_lower(
                    &resolved_type_env,
                    &common_root.module_name,
                    (func_name, return_type, parameters, stmts),
                )?;
                lower_functions.push(f);
            }
            high_ast::TopLevelDeclKind::Import { module_name: _ } => {}
            high_ast::TopLevelDeclKind::PubType {
                type_name: _,
                to: _,
            } => {}
        }
    }

    Ok(typed_ast::Root {
        functions: lower_functions,
        constants,
    })
}

/// high_ast::Function => typed_ast::Function
fn fn_to_lower(
    type_env: &HashMap<String, PeachiliType>,
    ref_module_name: &String,
    f_attrs: (
        &String,
        &String,
        &HashMap<String, String>,
        &[high_ast::Stmt],
    ),
) -> Result<typed_ast::Function, Box<dyn std::error::Error>> {
    let (fn_name, _return_type, params, stmts) = f_attrs;

    // 関数の返り値の型解決
    let mut local_variables: HashMap<String, typed_ast::FrameObject> = Default::default();
    let mut fn_stack_size = 0;

    // 引数リストを解決する
    let lower_params = {
        let mut lower_params: HashMap<String, PeachiliType> = Default::default();
        for (param_name, param_type_name) in params.iter() {
            let (param_type, _) = find_identifier_type_forcibly(
                type_env,
                &local_variables,
                ref_module_name,
                param_type_name,
            );
            fn_stack_size += param_type.size;

            local_variables.insert(
                param_name.to_string(),
                typed_ast::FrameObject {
                    stack_offset: fn_stack_size,
                    p_type: param_type.clone(),
                },
            );
            lower_params.insert(param_name.to_string(), param_type.clone());
        }
        lower_params
    };

    let mut lower_stmts = Vec::new();
    for stmt in stmts {
        let (stmt, lvs) = stmt_to_lower(
            stmt,
            local_variables,
            &type_env,
            ref_module_name,
            &mut fn_stack_size,
        );
        local_variables = lvs;
        lower_stmts.push(stmt);
    }

    Ok(typed_ast::Function {
        name: fn_name.to_string(),
        return_type: type_env.get(fn_name).unwrap().clone(),
        params: lower_params,
        local_variables,
        stack_size: fn_stack_size,
        stmts: lower_stmts,
    })
}

fn stmt_to_lower(
    stmt: &high_ast::Stmt,
    mut local_variables: HashMap<String, typed_ast::FrameObject>,
    type_env: &HashMap<String, peachili_type::PeachiliType>,
    ref_module_name: &str,
    stack_offset: &mut usize,
) -> (
    typed_ast::Statement,
    HashMap<String, typed_ast::FrameObject>,
) {
    match &stmt.kind {
        high_ast::StmtKind::Expr { expr: expr_info } => {
            let ex = expr_to_lower(
                &expr_info,
                &local_variables,
                type_env,
                ref_module_name,
                stack_offset,
            );
            (typed_ast::Statement::Expr { expr: ex }, local_variables)
        }
        high_ast::StmtKind::Declare {
            var_name,
            type_name,
        } => {
            let (id_ty, _) = find_identifier_type_forcibly(
                type_env,
                &local_variables,
                ref_module_name,
                &type_name.join("::"),
            );
            *stack_offset += id_ty.size;
            local_variables.insert(
                var_name.clone(),
                typed_ast::FrameObject {
                    stack_offset: *stack_offset,
                    p_type: id_ty.clone(),
                },
            );
            (typed_ast::Statement::Nop, local_variables)
        }
        high_ast::StmtKind::Asm { insts } => (
            typed_ast::Statement::Asm {
                insts: insts.clone(),
            },
            local_variables,
        ),
    }
}

fn expr_to_lower(
    expr: &high_ast::Expr,
    local_variables: &HashMap<String, typed_ast::FrameObject>,
    type_env: &HashMap<String, peachili_type::PeachiliType>,
    ref_module_name: &str,
    stack_offset: &mut usize,
) -> typed_ast::Expression {
    match &expr.kind {
        high_ast::ExprKind::True => typed_ast::Expression::new(
            typed_ast::ExprKind::True,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Boolean, 8),
        ),
        high_ast::ExprKind::False => typed_ast::Expression::new(
            typed_ast::ExprKind::False,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Boolean, 8),
        ),
        high_ast::ExprKind::Identifier { list } => {
            let (id_ty, offset) = find_identifier_type_forcibly(
                type_env,
                local_variables,
                ref_module_name,
                &list.join("::"),
            );
            typed_ast::Expression::new(
                typed_ast::ExprKind::Identifier {
                    list: list.clone(),
                    stack_offset: offset,
                },
                id_ty,
            )
        }
        high_ast::ExprKind::Integer { value } => typed_ast::Expression::new(
            typed_ast::ExprKind::Integer { value: *value },
            peachili_type::PeachiliType::new(peachili_type::PTKind::Int64, 8),
        ),
        high_ast::ExprKind::UnsignedInteger { value } => typed_ast::Expression::new(
            typed_ast::ExprKind::UnsignedInteger { value: *value },
            peachili_type::PeachiliType::new(peachili_type::PTKind::Uint64, 8),
        ),
        high_ast::ExprKind::Negative { child } => {
            let child_expr = expr_to_lower(
                &child.borrow(),
                local_variables,
                type_env,
                ref_module_name,
                stack_offset,
            );
            let neg_ty = child_expr.ty.clone();
            typed_ast::Expression::new(
                typed_ast::ExprKind::Negative {
                    child: Rc::new(RefCell::new(child_expr)),
                },
                neg_ty,
            )
        }
        high_ast::ExprKind::StringLiteral { contents } => {
            let mut s = DefaultHasher::new();
            contents.hash(&mut s);
            typed_ast::Expression::new(
                typed_ast::ExprKind::StringLiteral {
                    contents: contents.to_string(),
                    id: s.finish(),
                },
                peachili_type::PeachiliType::new(peachili_type::PTKind::ConstStr, 8),
            )
        }
        high_ast::ExprKind::Call { ident, params } => {
            let lower_ident = expr_to_lower(
                &ident.borrow(),
                local_variables,
                type_env,
                ref_module_name,
                stack_offset,
            );
            let mut lower_params = Vec::new();
            for param in params.iter() {
                lower_params.push(expr_to_lower(
                    param,
                    local_variables,
                    type_env,
                    ref_module_name,
                    stack_offset,
                ));
            }

            let call_ty = lower_ident.ty.clone();

            typed_ast::Expression::new(
                typed_ast::ExprKind::Call {
                    ident: match lower_ident.kind {
                        typed_ast::ExprKind::Identifier {
                            list,
                            stack_offset: _,
                        } => list.join("::"),
                        _ => unreachable!(),
                    },
                    params: lower_params,
                },
                call_ty,
            )
        }
        high_ast::ExprKind::Assignment {
            var_name,
            expr: var_expr,
        } => {
            let (id_ty, offset) =
                find_identifier_type_forcibly(type_env, local_variables, ref_module_name, var_name);
            let ex = expr_to_lower(
                &var_expr.as_ref().borrow(),
                local_variables,
                type_env,
                ref_module_name,
                stack_offset,
            );
            typed_ast::Expression::new(
                typed_ast::ExprKind::Assignment {
                    var_name: var_name.clone(),
                    var_stack_offset: offset,
                    expr: typed_ast::Expression::new_edge(ex),
                },
                id_ty,
            )
        }
        high_ast::ExprKind::Addition { lhs, rhs }
        | high_ast::ExprKind::Subtraction { lhs, rhs }
        | high_ast::ExprKind::Multiplication { lhs, rhs }
        | high_ast::ExprKind::Division { lhs, rhs } => binary_expr_to_lower(
            expr,
            (lhs, rhs),
            local_variables,
            type_env,
            ref_module_name,
            stack_offset,
        ),
    }
}

fn binary_expr_to_lower(
    ex: &high_ast::Expr,
    edges: (&Rc<RefCell<high_ast::Expr>>, &Rc<RefCell<high_ast::Expr>>),
    local_variables: &HashMap<String, typed_ast::FrameObject>,
    type_env: &HashMap<String, peachili_type::PeachiliType>,
    ref_module_name: &str,
    stack_offset: &mut usize,
) -> typed_ast::Expression {
    let (lhs, rhs) = edges;
    let lhs = expr_to_lower(
        &lhs.as_ref().borrow(),
        local_variables,
        type_env,
        ref_module_name,
        stack_offset,
    );
    let lhs_ty = lhs.ty;
    let rhs = expr_to_lower(
        &rhs.as_ref().borrow(),
        local_variables,
        type_env,
        ref_module_name,
        stack_offset,
    );
    match &ex.kind {
        high_ast::ExprKind::Addition { lhs: _, rhs: _ } => typed_ast::Expression::new(
            typed_ast::ExprKind::Addition {
                lhs: typed_ast::Expression::new_edge(lhs),
                rhs: typed_ast::Expression::new_edge(rhs),
            },
            peachili_type::PeachiliType::new(lhs_ty.kind, lhs_ty.size),
        ),
        high_ast::ExprKind::Subtraction { lhs: _, rhs: _ } => typed_ast::Expression::new(
            typed_ast::ExprKind::Subtraction {
                lhs: typed_ast::Expression::new_edge(lhs),
                rhs: typed_ast::Expression::new_edge(rhs),
            },
            peachili_type::PeachiliType::new(lhs_ty.kind, lhs_ty.size),
        ),

        high_ast::ExprKind::Multiplication { lhs: _, rhs: _ } => typed_ast::Expression::new(
            typed_ast::ExprKind::Multiplication {
                lhs: typed_ast::Expression::new_edge(lhs),
                rhs: typed_ast::Expression::new_edge(rhs),
            },
            peachili_type::PeachiliType::new(lhs_ty.kind, lhs_ty.size),
        ),
        high_ast::ExprKind::Division { lhs: _, rhs: _ } => typed_ast::Expression::new(
            typed_ast::ExprKind::Division {
                lhs: typed_ast::Expression::new_edge(lhs),
                rhs: typed_ast::Expression::new_edge(rhs),
            },
            peachili_type::PeachiliType::new(lhs_ty.kind, lhs_ty.size),
        ),
        _ => unreachable!(),
    }
}

fn find_identifier_type_forcibly(
    type_env: &HashMap<String, peachili_type::PeachiliType>,
    local_variables: &HashMap<String, typed_ast::FrameObject>,
    ref_module_name: &str,
    name: &str,
) -> (peachili_type::PeachiliType, usize) {
    match type_env.get(name) {
        Some(id_ty) => (id_ty.clone(), 0),
        None => match type_env.get(&format!("{}::{}", ref_module_name, name)) {
            Some(id_ty) => (id_ty.clone(), 0),
            None => match local_variables.get(name) {
                Some(obj) => (obj.p_type.clone(), obj.stack_offset),
                None => unreachable!(),
            },
        },
    }
}

fn evaluate_constant_expr(
    const_name: &String,
    const_expr: &high_ast::Expr,
) -> Result<typed_ast::Constant, ConstError> {
    match const_expr.kind {
        // 64bit整数の範囲を超えていたらとりあえずエラー
        high_ast::ExprKind::Integer { value } => {
            if value > (std::i64::MAX as i128) {
                Err(ConstError::CannotEvaluate {
                    name: const_name.to_string(),
                })
            } else {
                Ok(typed_ast::Constant::Integer(value as i64))
            }
        }
        high_ast::ExprKind::UnsignedInteger { value } => {
            if value > (std::u64::MAX as u128) {
                Err(ConstError::CannotEvaluate {
                    name: const_name.to_string(),
                })
            } else {
                Ok(typed_ast::Constant::UnsignedInteger(value as u64))
            }
        }
        _ => Err(ConstError::CannotEvaluate {
            name: const_name.to_string(),
        }),
    }
}
