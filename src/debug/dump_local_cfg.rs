use crate::common::three_address_code as tac;
use crate::common::cfg::LocalControlFlowGraph;
use crate::common::liveness_info::LocalLivenessInfo;
use crate::common::file_util;
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

/// 入り口生存，出口生存集合を含めたダンプ
pub fn dump_local_cfg_with_liveness(
    file_path: &str,
    ir_module: &tac::IRModule,
    local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>,
    liveness_info: &BTreeMap<tac::IRFunctionId, LocalLivenessInfo>,
) {
    let mut cfg_dumper = CFGDumper {
        output: String::new(),
        file_path: file_path.to_string(),
    };
    cfg_dumper.output += "digraph G { \n";

    for fn_id in ir_module.funcs.iter() {
        cfg_dumper.append_function_cluster_with_liveness(ir_module, fn_id, local_cfg, liveness_info);
    }

    cfg_dumper.output += "} \n";

    cfg_dumper.to_file();
}


impl CFGDumper {
    /// サブグラフの定義
    fn append_function_cluster(&mut self, ir_module: &tac::IRModule, ir_fn_id: &tac::IRFunctionId, local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>) {
        let ir_fn = ir_module.fn_allocator.get(*ir_fn_id).unwrap();

        self.append_cluster_attributes(&ir_fn.name);

        for code_id in ir_fn.codes.iter() {
            self.append_ir_node(*code_id, ir_fn);

            // エッジ定義
            self.append_succ_edge(local_cfg, ir_fn_id, code_id, &ir_fn.name);
        }

        self.output += "  }\n";
    }

    /// 生存情報を含めたサブグラフの定義
    fn append_function_cluster_with_liveness(
        &mut self,
        ir_module: &tac::IRModule,
        ir_fn_id: &tac::IRFunctionId,
        local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>,
        liveness_info: &BTreeMap<tac::IRFunctionId, LocalLivenessInfo>,
    ) {
        let ir_fn = ir_module.fn_allocator.get(*ir_fn_id).unwrap();

        self.append_cluster_attributes(&ir_fn.name);

        for code_id in ir_fn.codes.iter() {
            self.append_ir_node_with_liveness(*code_id, ir_fn, ir_fn_id, liveness_info);

            // エッジ定義
            self.append_succ_edge(local_cfg, ir_fn_id, code_id, &ir_fn.name);
        }

        self.output += "  }\n";
    }

    /// 生存情報を含めたノード定義
    fn append_ir_node_with_liveness(
        &mut self,
        code_id: tac::CodeId,
        ir_fn: &tac::IRFunction,
        ir_fn_id: &tac::IRFunctionId,
        liveness_info: &BTreeMap<tac::IRFunctionId, LocalLivenessInfo>,
    ) {
        // ノード定義
        let (code, shape) = self.get_shape(ir_fn, &code_id);

        let mut live_in = Vec::new();
        for v_id in liveness_info.get(ir_fn_id).unwrap().get_live_in(&code_id).iter(){
            live_in.push(ir_fn.get_value(*v_id).dump());
        }
        let mut live_out = Vec::new();

        for v_id in liveness_info.get(ir_fn_id).unwrap().get_live_out(&code_id).iter(){
            live_out.push(ir_fn.get_value(*v_id).dump());
        }


        self.output +=
            &format!(
                "    \"{}{:?}\"[label=\"{}\nlive_in {{ {} }}\nlive_out {{ {} }}\", shape=\"{}\"]\n",
                ir_fn.name,
                code_id,
                code.dump(ir_fn.value_allocator.clone()),
                live_in.join(", "),
                live_out.join(","),
                shape
            );
    }

    /// グラフのノード定義
    fn append_ir_node(&mut self, code_id: tac::CodeId, ir_fn: &tac::IRFunction) {
        let (code, shape) = self.get_shape(ir_fn, &code_id);
        self.output += &format!("    \"{}{:?}\"[label=\"{}\", shape=\"{}\"]\n", ir_fn.name, code_id, code.dump(ir_fn.value_allocator.clone()), shape);
    }

    /// ノードの形をCodeKindによって決める
    fn get_shape(&self, ir_fn: &tac::IRFunction, code_id: &tac::CodeId) -> (tac::Code, String) {
        let code = ir_fn.code_allocator.lock().unwrap().get(*code_id).unwrap().clone();
        // ノード定義
        let shape = match code.kind {
            tac::CodeKind::LABEL { name: _ } => {
                "ellipse"
            }
            _ => {
                "box"
            }
        };
        (code, shape.to_string())
    }

    /// 後続節の定義
    fn append_succ_edge(&mut self, local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>, ir_fn_id: &tac::IRFunctionId, code_id: &tac::CodeId, cluster_name: &str) {
        if let Some(graph) = local_cfg.get(ir_fn_id).unwrap().successors.get(code_id) {
            for succ_edge in graph.iter() {
                self.output += &format!("    \"{}{:?}\" -> \"{}{:?}\"\n", cluster_name, code_id, cluster_name, succ_edge);
            }
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