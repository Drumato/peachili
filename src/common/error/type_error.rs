use crate::common::error::CompileErrorKind;
use fmt::Formatter;
use std::fmt;

use crate::common::ast;

/// Analyzerが発行するエラーを格納
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TypeError {
    /// エラーの種類
    kind: TypeErrorKind,
}

/// Analyzerが発行するエラーの種類を列挙
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TypeErrorKind {
    /// 型を導出しきれなかった
    CannotResolve { type_name: String },

    /// メイン関数が見つからなかった
    NotFoundMainFunction,

    /// メイン関数に何らかの引数が定義されてしまっている
    MAINFUNCMUSTNOTHAVEANYARGUMENTS,

    /// メイン関数はいかなる値も返さない
    MainFunctionMustNotReturnAnyValues,

    /// 型名の場所で関数名が使用された
    GotFunctionNameAsType { func_name: String },
    /// 型名の場所で定数名が使用された
    GotConstantNameAsType { const_name: String },

    /// 変数以外へメンバアクセスしようとした．
    CannotAccessMemberWithNotAnIdentifier { struct_node: ast::ExpressionNode },

    /// 構造体型以外にメンバアクセスした
    CannotAccessMemberWIthNotAStruct { struct_node: ast::ExpressionNode },

    /// 該当するメンバが存在しなかった
    UndefinedSuchAMember { member: String },
}

impl CompileErrorKind for TypeErrorKind {
    fn category(&self) -> &'static str {
        "TypeError"
    }
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TypeErrorKind::CannotResolve { type_name } => {
                format!("cannot resolve a type -> `{}`", type_name)
            }
            TypeErrorKind::GotFunctionNameAsType { func_name } => {
                format!("a function `{}` used as a type-name", func_name)
            }
            TypeErrorKind::GotConstantNameAsType { const_name } => {
                format!("a constant `{}` used as a type-name", const_name)
            }
            TypeErrorKind::CannotAccessMemberWithNotAnIdentifier { struct_node } => format!(
                "cannot access member of `{:?}`, its not an identifier",
                struct_node
            ),
            TypeErrorKind::CannotAccessMemberWIthNotAStruct { struct_node } => format!(
                "cannot access member of `{:?}`, its not a struct",
                struct_node
            ),
            TypeErrorKind::UndefinedSuchAMember { member } => {
                format!("undefined such a member -> `{}`", member)
            }
            TypeErrorKind::NotFoundMainFunction => "entry point `main` not found".to_string(),
            TypeErrorKind::MAINFUNCMUSTNOTHAVEANYARGUMENTS => {
                "entry point `main` mustn't have any arguments".to_string()
            }
            TypeErrorKind::MainFunctionMustNotReturnAnyValues => {
                "entry point `main` mustn't return any values".to_string()
            }
        };

        write!(f, "{}", s)
    }
}
