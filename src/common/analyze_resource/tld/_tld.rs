use crate::common::ast;
use crate::common::tld::tld_kind;

/// 宣言
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct TopLevelDecl {
    pub kind: tld_kind::TLDKind,
}

impl TopLevelDecl {
    pub fn new(k: tld_kind::TLDKind) -> Self {
        Self { kind: k }
    }
    pub fn new_alias(src_type: &str) -> Self {
        Self::new(tld_kind::TLDKind::ALIAS {
            src_type: src_type.to_string(),
        })
    }
    pub fn new_const(type_name: &str, expr: String) -> Self {
        Self::new(tld_kind::TLDKind::CONST {
            type_name: type_name.to_string(),
            expr,
        })
    }

    pub fn new_function_from_ast(fn_ty: ast::FunctionTypeDef) -> Self {
        Self::new(tld_kind::TLDKind::FN {
            return_type: fn_ty.return_type,
            args: fn_ty.args,
        })
    }

    pub fn new_struct_from_ast(st_ty: ast::StructDef) -> Self {
        Self::new(tld_kind::TLDKind::STRUCT {
            members: st_ty.members,
        })
    }

    pub fn new_enum(en_ty: ast::EnumDef) -> Self {
        Self::new(tld_kind::TLDKind::ENUM {
            variants: en_ty
                .variants
                .iter()
                .map(|(name, variant)| (name.to_string(), variant.tag))
                .collect(),
        })
    }
}
