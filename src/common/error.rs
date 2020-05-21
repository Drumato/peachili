extern crate colored;

use colored::*;

use crate::common::{option as opt, position as pos};
use crate::compiler::resource as res;

#[derive(Clone)]
pub struct CompileError {
    kind: CmpErrorKind,
    position: pos::Position,
}

impl CompileError {
    fn new(err_kind: CmpErrorKind, err_pos: pos::Position) -> Self {
        Self {
            kind: err_kind,
            position: err_pos,
        }
    }

    fn format_with(&self, module_path: &str, message: String) -> String {
        format!(
            "{}{}: [{}] {}.",
            module_path,
            self.position,
            "CompileError".red().bold(),
            message
        )
    }

    pub fn emit_stderr(&self, module_path: &str, build_opt: &opt::BuildOption) {
        let message = match build_opt.language {
            opt::Language::JAPANESE => self.error_message_ja(),
            opt::Language::ENGLISH => self.error_message_en(),
        };

        eprintln!("{}", self.format_with(module_path, message));
    }

    pub fn error_message_en(&self) -> String {
        match &self.kind {
            CmpErrorKind::OUTOF64BITSINTRANGE(number_str) => format!(
                "'{}' is bigger than 64-bit signed integer's limit",
                number_str
            ),
            CmpErrorKind::CALLEDUNDEFINEDFUNCTION(called_name) => {
                format!("the '{}' function is not defined", called_name)
            }
            CmpErrorKind::USEDUNDEFINEDAUTOVAR(ident_name) => {
                format!("the '{}' auto-variable is not defined", ident_name)
            }
            CmpErrorKind::UNABLETORESOLVEEXPRESSIONTYPE(expr) => {
                format!("unable to resolve an expression -> '{}'", expr)
            }
            CmpErrorKind::CONDITIONEXPRESSIONMUSTBEANBOOLEANINIF(expr_type) => format!(
                "an conditional-expression must be an boolean, but got'{}'",
                expr_type
            ),
            CmpErrorKind::CANNOTASSIGNMENTUNRESOLVEDRIGHTVALUE(lvalue, rvalue) => format!(
                "cannot assign an unresolved expression '{}'  into '{}'",
                rvalue, lvalue
            ),
            CmpErrorKind::BOTHVALUESMUSTBESAMETYPEINASSIGNMENT(lvalue, rvalue) => format!(
                "both value must be same type in assignment -> `{} = {}`",
                lvalue, rvalue
            ),
            CmpErrorKind::BOTHBLOCKSMUSTBESAMETYPE => {
                "if-block and else-block must have same type".to_string()
            }
            CmpErrorKind::BINARYOPERATIONMUSTHAVETOSAMETYPEOPERANDS(
                operator,
                lop_type,
                rop_type,
            ) => format!(
                "a binary operation '{}' has two different-type operands -> '{}, {}'",
                operator, lop_type, rop_type
            ),
            CmpErrorKind::CANNOTASSIGNMENTTOCONSTANTAFTERINITIALIZATION(const_var) => format!(
                "unable to assign the expression to the constant '{}' is already initialized",
                const_var
            ),
        }
    }
    pub fn error_message_ja(&self) -> String {
        match &self.kind {
            CmpErrorKind::OUTOF64BITSINTRANGE(number_str) => format!(
                "数値リテラル '{}' は64bit符号付き整数で表現できる範囲を超えています",
                number_str
            ),
            CmpErrorKind::CALLEDUNDEFINEDFUNCTION(called_name) => {
                format!("関数 '{}' は未定義です", called_name)
            }
            CmpErrorKind::USEDUNDEFINEDAUTOVAR(ident_name) => {
                format!("自動変数 '{}' は未定義です", ident_name,)
            }
            CmpErrorKind::UNABLETORESOLVEEXPRESSIONTYPE(expr) => {
                format!("式 '{}' の型を解決できません", expr)
            }
            CmpErrorKind::CONDITIONEXPRESSIONMUSTBEANBOOLEANINIF(expr_type) => format!(
                "if式の条件式ではbooleanしか用いることができませんが，'{}' 型を検出しました",
                expr_type
            ),
            CmpErrorKind::CANNOTASSIGNMENTUNRESOLVEDRIGHTVALUE(lvalue, rvalue) => format!(
                "型が解決できていない式 '{}' を左辺値 '{}' に代入することはできません",
                rvalue, lvalue
            ),
            CmpErrorKind::BOTHVALUESMUSTBESAMETYPEINASSIGNMENT(lvalue, rvalue) => format!(
                "代入式の両辺で異なる型を検知しました -> `{} = {}`",
                lvalue, rvalue
            ),
            CmpErrorKind::BOTHBLOCKSMUSTBESAMETYPE => {
                "if-blockとelse-blockは同じ型を持つ必要があります".to_string()
            }
            CmpErrorKind::BINARYOPERATIONMUSTHAVETOSAMETYPEOPERANDS(
                operator,
                lop_type,
                rop_type,
            ) => format!(
                "二項演算 '{}' の左右オペランドが異なる型 '{}, {}' を持っています",
                operator, lop_type, rop_type
            ),

            CmpErrorKind::CANNOTASSIGNMENTTOCONSTANTAFTERINITIALIZATION(const_var) => {
                format!("初期化されている定数 '{}' への代入文は無効です", const_var)
            }
        }
    }

    pub fn out_of_64bit_sint_range(number_str: String, err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::OUTOF64BITSINTRANGE(number_str), err_pos)
    }

    pub fn used_undefined_auto_var(ident_name: String, err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::USEDUNDEFINEDAUTOVAR(ident_name), err_pos)
    }

    pub fn called_undefined_function(called_name: String, err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::CALLEDUNDEFINEDFUNCTION(called_name), err_pos)
    }

    pub fn unable_to_resolve_expression_type(
        expr: res::ExpressionNode,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(CmpErrorKind::UNABLETORESOLVEEXPRESSIONTYPE(expr), err_pos)
    }

    pub fn condition_expression_must_be_an_boolean_in_if(
        expr_type: res::PType,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(
            CmpErrorKind::CONDITIONEXPRESSIONMUSTBEANBOOLEANINIF(expr_type),
            err_pos,
        )
    }

    pub fn cannot_assignment_unresolved_right_value(
        lvalue_type: res::ExpressionNode,
        rvalue: res::ExpressionNode,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(
            CmpErrorKind::CANNOTASSIGNMENTUNRESOLVEDRIGHTVALUE(lvalue_type, rvalue),
            err_pos,
        )
    }

    pub fn both_values_must_be_same_type_in_assignment(
        lvalue_type: res::PType,
        rvalue_type: res::PType,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(
            CmpErrorKind::BOTHVALUESMUSTBESAMETYPEINASSIGNMENT(lvalue_type, rvalue_type),
            err_pos,
        )
    }

    pub fn cannot_assignment_to_constant_after_initialization(
        lvalue: res::ExpressionNode,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(
            CmpErrorKind::CANNOTASSIGNMENTTOCONSTANTAFTERINITIALIZATION(lvalue),
            err_pos,
        )
    }

    pub fn both_blocks_must_be_same_type(err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::BOTHBLOCKSMUSTBESAMETYPE, err_pos)
    }

    pub fn binary_operation_must_have_two_same_type_operands(
        operator: String,
        lop_type: res::PType,
        rop_type: res::PType,
        err_pos: pos::Position,
    ) -> Self {
        Self::new(
            CmpErrorKind::BINARYOPERATIONMUSTHAVETOSAMETYPEOPERANDS(operator, lop_type, rop_type),
            err_pos,
        )
    }
}

#[derive(Clone)]
pub enum CmpErrorKind {
    /// 64bitより大きかった数字
    /// kind.0 -> 数値化できなかった数字列
    OUTOF64BITSINTRANGE(String),

    /// 定義されていない自動変数の使用
    /// kind.0 -> 未定義の変数名
    USEDUNDEFINEDAUTOVAR(String),

    /// 定義されていない関数のコール
    /// kind.0 -> 未定義の関数名
    CALLEDUNDEFINEDFUNCTION(String),

    /// 式の型を解決できなかった場合
    /// kind.0 -> 型を解決できなかった式
    UNABLETORESOLVEEXPRESSIONTYPE(res::ExpressionNode),

    /// IF式内の条件式がBooleanタイプでなかった
    /// kind.0 -> Boolean以外の型
    CONDITIONEXPRESSIONMUSTBEANBOOLEANINIF(res::PType),

    /// 型が解決できなかった右辺値を左辺値に代入しようとした
    /// kind.0 -> 左辺値
    /// kind.1 -> 右辺値
    CANNOTASSIGNMENTUNRESOLVEDRIGHTVALUE(res::ExpressionNode, res::ExpressionNode),

    /// 代入式が両辺で異なる型を持っている場合
    /// kind.0 -> 左辺値
    /// kind.1 -> 右辺値
    BOTHVALUESMUSTBESAMETYPEINASSIGNMENT(res::PType, res::PType),

    /// IF-ELSE式の2つのブロックの型が異なる場合
    BOTHBLOCKSMUSTBESAMETYPE,

    /// 二項演算の左右オペランドで型が異なる場合
    /// kind.0 -> 二項演算子
    /// kind.1 -> 左項
    /// kind.2 -> 右項
    BINARYOPERATIONMUSTHAVETOSAMETYPEOPERANDS(String, res::PType, res::PType),

    /// 定数への再代入
    /// kind.0 -> 定数ノード
    CANNOTASSIGNMENTTOCONSTANTAFTERINITIALIZATION(res::ExpressionNode),
}
