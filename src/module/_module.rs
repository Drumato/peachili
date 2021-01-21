use std::{cell::RefCell, path::PathBuf, rc::Rc};

/// 各ファイル(パッケージ)を表す構造体
/// 依存グラフの各ノードとしても動作する
#[derive(Clone)]
pub struct ModuleInfo<'a> {
    /// モジュールの種類
    pub kind: ModuleKind<'a>,
    /// モジュールが存在するパス
    pub file_path: PathBuf,
    /// モジュール名
    pub name: String,
}

pub type Module<'a> = &'a ModuleInfo<'a>;

#[allow(dead_code)]
impl<'a> ModuleInfo<'a> {
    fn new(kind: ModuleKind<'a>, file_path: PathBuf, name: String) -> Self {
        Self {
            kind,
            file_path,
            name,
        }
    }

    pub fn new_primitive(file_path: PathBuf, name: String, contents: String) -> Self {
        Self::new(
            ModuleKind::Primitive {
                contents,
                refs: Rc::new(RefCell::new(Default::default())),
            },
            file_path,
            name.replace("/", "::"),
        )
    }

    /// 外部パッケージを割り当てる
    pub fn new_directory(file_path: PathBuf, name: String) -> Self {
        Self::new(
            ModuleKind::Directory {
                children: Rc::new(RefCell::new(Vec::new())),
            },
            file_path,
            name.replace("/", "::"),
        )
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ModuleKind<'a> {
    /// ソースコードが含まれるファイルを表す
    Primitive {
        /// 参照するモジュール
        refs: Rc<RefCell<Vec<Module<'a>>>>,
        /// ソースコードの内容
        contents: String,
    },

    /// 複数のサブパッケージを含むディレクトリを表す
    Directory {
        /// ディレクトリにぶら下がっているモジュール
        children: Rc<RefCell<Vec<Module<'a>>>>,
    },
}
