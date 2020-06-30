extern crate colored;

use colored::*;

use crate::common::{option as opt, position as pos};
use crate::compiler::general::resource as res;

#[derive(Clone)]
pub struct CompileError {
    kind: CmpErrorKind,
    position: pos::Position,
}

impl CompileError {
    pub fn new(err_kind: CmpErrorKind, err_pos: pos::Position) -> Self {
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
            CmpErrorKind::ARGTYPEINCORRECT(fn_name, idx, expected, actual) => format!(
                "{}[{}] must be '{}' type，but got '{}'",
                fn_name, idx, expected, actual
            ),
            CmpErrorKind::ARGNUMBERINCORRECT(fn_name, expected, actual) => format!(
                "the function {} takes {} arguments but {} were suplied",
                fn_name, expected, actual
            ),
            CmpErrorKind::CANNOTAPPLYMINUSTO(value_type) => {
                format!("cannot apply unary operator `-` to type `{}`", value_type)
            }
            CmpErrorKind::RETURNINNORETURNFUNC => {
                "detect returning value in noreturn function".to_string()
            }
            CmpErrorKind::MAINMUSTEXIST => "main function must be declared".to_string(),
            CmpErrorKind::MAINMUSTHAVENOARGSANDNORETURN => {
                "main function's signature must be `main() noreturn`".to_string()
            }
            CmpErrorKind::STATEMENTMUSTBEENDEDWITHSEMICOLON => {
                "statement must be ended with `;`".to_string()
            }
            CmpErrorKind::EXPECTEDIDENTIFIER => "expected identifier".to_string(),
            CmpErrorKind::GOTINVALIDPTYPE => "got invalid type".to_string(),
            CmpErrorKind::CANNOTDEREFERENCENOTPOINTER(inner) => {
                format!("cannot dereference `{}`; its not a pointer", inner)
            }
            CmpErrorKind::RETURNLOCALADDRESS => {
                "unable to return an address of stack area".to_string()
            }
            CmpErrorKind::DEFINITIONMUSTHAVENAME => {
                "definition identifier must have one name".to_string()
            }
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
            CmpErrorKind::ARGTYPEINCORRECT(fn_name, idx, expected, actual) => format!(
                "{}[{}] は '{}' 型で定義されていますが， '{}' 型が渡されました",
                fn_name, idx, expected, actual
            ),
            CmpErrorKind::ARGNUMBERINCORRECT(fn_name, expected, actual) => format!(
                "関数 {} は {} つの引数が定義されていますが，{} つ渡されて呼び出されました",
                fn_name, expected, actual
            ),
            CmpErrorKind::CANNOTAPPLYMINUSTO(value_type) => format!(
                "単項演算子 `-` を型 `{}` に適用することはできません",
                value_type
            ),
            CmpErrorKind::RETURNINNORETURNFUNC => {
                "noreturn 型と定義された関数内で return statement を検知しました".to_string()
            }
            CmpErrorKind::MAINMUSTEXIST => {
                "main 関数は必ず定義されていなければなりません".to_string()
            }
            CmpErrorKind::MAINMUSTHAVENOARGSANDNORETURN => {
                "main関数の型シグネチャは必ず `main() noreturn` でなければなりません".to_string()
            }
            CmpErrorKind::STATEMENTMUSTBEENDEDWITHSEMICOLON => {
                "文は必ず `;` で終わる必要があります".to_string()
            }
            CmpErrorKind::EXPECTEDIDENTIFIER => "識別子以外を検知しました".to_string(),
            CmpErrorKind::GOTINVALIDPTYPE => "INVALID 型を検知しました".to_string(),
            CmpErrorKind::CANNOTDEREFERENCENOTPOINTER(inner) => format!(
                "非ポインタ型である `{}` に対してデリファレンスは無効です",
                inner
            ),
            CmpErrorKind::RETURNLOCALADDRESS => {
                "関数フレームが開放される危険があるため，ローカル変数のアドレスを返すのは禁止です"
                    .to_string()
            }
            CmpErrorKind::DEFINITIONMUSTHAVENAME => {
                "識別子を定義する際には名前が必要です".to_string()
            }
        }
    }

    pub fn out_of_64bit_sint_range(number_str: String, err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::OUTOF64BITSINTRANGE(number_str), err_pos)
    }

}

#[derive(Clone)]
pub enum CmpErrorKind {
    // Parse Errors
    /// 文は必ずセミコロンで終わる必要がある．
    STATEMENTMUSTBEENDEDWITHSEMICOLON,

    /// 識別子が必要な場面
    EXPECTEDIDENTIFIER,

    /// invalid型の受け取り
    GOTINVALIDPTYPE,

    // Semantic Errors
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

    /// 引数の型の不一致
    /// kind.0 -> 呼び出された関数名
    /// kind.1 -> 引数の場所
    /// kind.2 -> 定義された型
    /// kind.3 -> 実際に渡された型
    ARGTYPEINCORRECT(String, usize, res::PType, res::PType),

    /// 引数の数の不一致
    /// kind.0 -> 呼び出された関数名
    /// kind.1 -> 定義された引数の数
    /// kind.2 -> 渡された引数の数
    ARGNUMBERINCORRECT(String, usize, usize),

    /// マイナス演算子を適用できない型
    /// kind.0 -> 演算子を適用した型名
    CANNOTAPPLYMINUSTO(res::PType),

    /// noreturn が返り値となっている関数内でreturn文
    RETURNINNORETURNFUNC,

    /// main関数は必ず定義されていなければならない
    MAINMUSTEXIST,

    /// main関数の型シグネチャは固定
    MAINMUSTHAVENOARGSANDNORETURN,

    /// 非ポインタ型に対するデリファレンス
    /// kind.0 -> 非ポインタ型
    CANNOTDEREFERENCENOTPOINTER(res::PType),

    /// ローカル変数のポインタを返している
    RETURNLOCALADDRESS,

    /// 定義時に識別子名を記述していない
    DEFINITIONMUSTHAVENAME,
}
