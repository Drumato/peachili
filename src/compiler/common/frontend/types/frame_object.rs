use std::collections::HashMap;

use super::peachili_type;

pub struct FrameObject {
    pub stack_offset: usize,
    pub p_type: peachili_type::PeachiliType,
}

/// グローバルな識別子の情報を格納しておく
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalEnv {
    /// 関数とそれに対応する型を格納
    /// 現在は単に返り値の型だけ持っておく
    pub func_table: HashMap<String, peachili_type::PeachiliType>,
    /// ユーザ定義の型名等を解決するテーブル
    pub type_name_table: HashMap<String, peachili_type::PeachiliType>,
}

impl Default for GlobalEnv {
    fn default() -> Self {
        Self {
            func_table: Default::default(),
            type_name_table: Default::default(),
        }
    }
}

impl GlobalEnv {
    // 言語が組み込みで使用する型名の定義
    // targetを受け取るようにしてサイズを変更していったほうが良い
    pub fn initialize_predefined_type(&mut self) {
        self.type_name_table.insert(
            "Int64".to_string(),
            peachili_type::PeachiliType::new(peachili_type::PTKind::Int64, 8),
        );
        self.type_name_table.insert(
            "Uint64".to_string(),
            peachili_type::PeachiliType::new(peachili_type::PTKind::Uint64, 8),
        );
        self.type_name_table.insert(
            "Noreturn".to_string(),
            peachili_type::PeachiliType::new(peachili_type::PTKind::Noreturn, 0),
        );
        self.type_name_table.insert(
            "ConstStr".to_string(),
            peachili_type::PeachiliType::new(peachili_type::PTKind::Noreturn, 0),
        );
        self.type_name_table.insert(
            "Boolean".to_string(),
            peachili_type::PeachiliType::new(peachili_type::PTKind::Boolean, 0),
        );
    }
}
