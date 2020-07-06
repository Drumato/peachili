use crate::common::{
    module,
    compiler,
};

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn main(main_module_id: module::ModuleId) {

    // TODO: ASTのProgram構造体を返すようにする
    compiler::frontend(main_module_id);
}