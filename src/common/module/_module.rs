use std::sync::{Arc, Mutex};

/// 各ファイル(パッケージ)を表す構造体
/// 依存グラフの各ノードとしても動作する
#[derive(Clone)]
pub struct ModuleData<'a> {
    /// モジュールの種類
    pub kind: ModuleKind,
    /// 参照するモジュール
    pub refs: Arc<Mutex<Vec<Module<'a>>>>,
    /// ディレクトリにぶら下がっているモジュール
    pub childs: Arc<Mutex<Vec<Module<'a>>>>,
    /// モジュールが存在するパス
    file_path: String,
    /// モジュール名
    name: String,
}

pub type Module<'a> = &'a ModuleData<'a>;

#[allow(dead_code)]
impl<'a> ModuleData<'a> {
    fn new(kind: ModuleKind, file_path: String, name: String) -> Self {
        Self {
            kind,
            file_path,
            name,
            refs: Arc::new(Mutex::new(Vec::new())),
            childs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// モジュールの依存ノードを追加する
    pub fn add_reference_module(&self, ref_module: Module<'a>) {
        self.refs.lock().unwrap().push(ref_module);
    }

    /// モジュールの下位ノードを追加する
    pub fn add_child_module(&self, child_module: Module<'a>) {
        self.childs.lock().unwrap().push(child_module);
    }

    /// ファイルパスの参照
    pub fn get_path(&self) -> &String {
        &self.file_path
    }

    /// ファイルパスのコピー
    pub fn copy_path(&self) -> String {
        self.file_path.to_string()
    }

    /// mainパッケージを割り当てる
    pub fn new_primary(file_path: String, name: String) -> Self {
        Self::new(ModuleKind::PRIMARY, file_path, name)
    }

    /// 外部パッケージを割り当てる
    pub fn new_external(file_path: String, name: String) -> Self {
        Self::new(ModuleKind::EXTERNAL, file_path, name)
    }

    /// ファイルパスの設定
    pub fn set_file_path(&mut self, fp: String) {
        self.file_path = fp;
    }

    /// 依存モジュール数の取得
    pub fn ref_count(&self) -> usize {
        self.refs.lock().unwrap().len()
    }

    /// 下位モジュール数の取得
    pub fn child_count(&self) -> usize {
        self.childs.lock().unwrap().len()
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ModuleKind {
    /// func main() Noreturn を持つファイルのみが該当
    /// このパッケージが他のパッケージから参照されることはない
    PRIMARY,

    /// 何らかのパッケージから参照されているパッケージ
    EXTERNAL,
}
