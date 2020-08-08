use crate::common::cfg::LocalControlFlowGraph;
use crate::common::file_util;
use crate::common::three_address_code as tac;
use std::collections::BTreeMap;

struct CFGDumper {
    output: String,
    file_path: String,
}

/// 通常のCFGダンプ
pub fn dump_local_cfg(
    file_path: &str,
    ir_module: &tac::IRModule,
    local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>,
) {
    let mut cfg_dumper = CFGDumper {
        output: String::new(),
        file_path: file_path.to_string(),
    };
    cfg_dumper.output += "digraph G { \n";

    for fn_id in ir_module.funcs.iter() {
        cfg_dumper.append_function_cluster(ir_module, fn_id, local_cfg);
    }

    cfg_dumper.output += "} \n";

    cfg_dumper.to_file();
}

impl CFGDumper {
    /// サブグラフの定義
    fn append_function_cluster(
        &mut self,
        ir_module: &tac::IRModule,
        ir_fn_id: &tac::IRFunctionId,
        local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>,
    ) {
        let ir_fn = ir_module.fn_allocator.get(*ir_fn_id).unwrap();

        self.append_cluster_attributes(&ir_fn.name);

        for code_id in ir_fn.codes.iter() {
            self.append_ir_node(*code_id, ir_fn);

            // エッジ定義
            self.append_succ_edge(local_cfg, ir_fn_id, code_id, &ir_fn.name);
        }

        self.output += "  }\n";
    }

    /// グラフのノード定義
    fn append_ir_node(&mut self, code_id: tac::CodeId, ir_fn: &tac::IRFunction) {
        let (code, shape) = self.get_shape(ir_fn, &code_id);
        self.output += &format!(
            "    \"{}{:?}\"[label=\"{}\", shape=\"{}\"]\n",
            ir_fn.name,
            code_id,
            code.dump(ir_fn.value_allocator.clone()),
            shape
        );
    }

    /// ノードの形をCodeKindによって決める
    fn get_shape(&self, ir_fn: &tac::IRFunction, code_id: &tac::CodeId) -> (tac::Code, String) {
        let code = ir_fn.get_code(*code_id);
        // ノード定義
        let shape = match code.kind {
            tac::CodeKind::LABEL { name: _ } => "ellipse",
            _ => "box",
        };
        (code, shape.to_string())
    }

    /// 後続節の定義
    fn append_succ_edge(
        &mut self,
        local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>,
        ir_fn_id: &tac::IRFunctionId,
        code_id: &tac::CodeId,
        cluster_name: &str,
    ) {
        let graph = local_cfg.get(ir_fn_id).unwrap().get_successors(code_id);
        for succ_edge in graph.iter() {
            self.output += &format!(
                "    \"{}{:?}\" -> \"{}{:?}\"\n",
                cluster_name, code_id, cluster_name, succ_edge
            );
        }
    }

    /// サブグラフの情報
    fn append_cluster_attributes(&mut self, cluster_name: &str) {
        self.output += &format!("  subgraph cluster_{} {{\n", cluster_name);
        self.output += &format!("    label = \"{}\";\n", cluster_name);
        self.output += "    labelloc = \"t\"\n";
        self.output += "    labeljust = \"l\"\n";
        self.output += "    fillcolor = \"#ababab\";\n";
    }

    fn to_file(&self) {
        file_util::write_program_into(&self.file_path, self.output.to_string());
    }
}
