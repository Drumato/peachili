use std::sync::{Arc, Mutex};
use std::path::PathBuf;

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
    /// 参照するモジュール
    pub refs: Arc<Mutex<Vec<Module<'a>>>>,
}

pub type Module<'a> = &'a ModuleInfo<'a>;

#[allow(dead_code)]
impl<'a> ModuleInfo<'a> {
    fn new(kind: ModuleKind<'a>, file_path: PathBuf, name: String) -> Self {
        Self {
            kind,
            file_path,
            name,
            refs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// mainパッケージを割り当てる
    pub fn new_primary(file_path: PathBuf, name: String) -> Self {
        Self::new(ModuleKind::Primary, file_path, name)
    }

    /// 外部パッケージを割り当てる
    pub fn new_external(file_path: PathBuf, name: String) -> Self {
        Self::new(
            ModuleKind::External {
                children: Arc::new(Mutex::new(Vec::new())),
            },
            file_path,
            name,
        )
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ModuleKind<'a> {
    /// func main() Noreturn を持つファイルのみが該当
    /// このパッケージが他のパッケージから参照されることはない
    Primary,

    /// 何らかのパッケージから参照されているパッケージ
    External {
        /// ディレクトリにぶら下がっているモジュール
        children: Arc<Mutex<Vec<Module<'a>>>>,
    },
}
