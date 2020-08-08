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
    CANNOTRESOLVE { type_name: String },

    /// メイン関数が見つからなかった
    MAINFUNCNOTFOUND,

    /// メイン関数に何らかの引数が定義されてしまっている
    MAINFUNCMUSTNOTHAVEANYARGUMENTS,

    /// メイン関数はいかなる値も返さない
    MAINFUNCMUSTNOTRETURNANYVALUES,

    /// 型名の場所で関数名が使用された
    GOTFUNCTIONNAMEASTYPE { func_name: String },

    /// 変数以外へメンバアクセスしようとした．
    CANNOTACCESSMEMBERWITHNOTANIDENTIFIER { struct_node: ast::ExpressionNode },

    /// メンバ名が識別子でなかった
    MEMBERNAMEMUSTBEANIDENTIFIER { member_node: ast::ExpressionNode },

    /// 構造体型以外にメンバアクセスした
    CANNOTACCESSMEMBERWITHNOTASTRUCT { struct_node: ast::ExpressionNode },

    /// 該当するメンバが存在しなかった
    UNDEFINEDSUCHAMEMBER { member_node: ast::ExpressionNode },
}

impl CompileErrorKind for TypeErrorKind {
    fn category(&self) -> &'static str {
        "TypeError"
    }
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TypeErrorKind::CANNOTRESOLVE { type_name } => {
                format!("cannot resolve a type -> `{}`", type_name)
            }
            TypeErrorKind::GOTFUNCTIONNAMEASTYPE { func_name } => {
                format!("a function `{}` used as a type-name", func_name)
            }
            TypeErrorKind::CANNOTACCESSMEMBERWITHNOTANIDENTIFIER { struct_node } => format!(
                "cannot access member of `{:?}`, its not an identifier",
                struct_node
            ),
            TypeErrorKind::MEMBERNAMEMUSTBEANIDENTIFIER { member_node } => format!(
                "member name must be an identifier, but got `{:?}",
                member_node
            ),
            TypeErrorKind::CANNOTACCESSMEMBERWITHNOTASTRUCT { struct_node } => format!(
                "cannot access member of `{:?}`, its not a struct",
                struct_node
            ),
            TypeErrorKind::UNDEFINEDSUCHAMEMBER { member_node } => {
                format!("undefined such a member -> `{:?}`", member_node)
            }
            TypeErrorKind::MAINFUNCNOTFOUND => "entry point `main` not found".to_string(),
            TypeErrorKind::MAINFUNCMUSTNOTHAVEANYARGUMENTS => {
                "entry point `main` mustn't have any arguments".to_string()
            }
            TypeErrorKind::MAINFUNCMUSTNOTRETURNANYVALUES => {
                "entry point `main` mustn't return any values".to_string()
            }
        };

        write!(f, "{}", s)
    }
}
