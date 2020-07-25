use crate::common::three_address_code as tac;
use crate::common::cfg::LocalControlFlowGraph;
use crate::common::file_util;
use std::collections::BTreeMap;

struct CFGDumper {
    output: String,
    file_path: String,
}

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
    fn append_function_cluster(&mut self, ir_module: &tac::IRModule, ir_fn_id: &tac::IRFunctionId, local_cfg: &BTreeMap<tac::IRFunctionId, LocalControlFlowGraph>) {
        let ir_fn = ir_module.fn_allocator.get(*ir_fn_id).unwrap();

        self.output += &format!("  subgraph cluster_{} {{\n", ir_fn.name);
        self.output += &format!("    label = \"{}\";\n", ir_fn.name);
        self.output += "    labelloc = \"t\"\n";
        self.output += "    labeljust = \"l\"\n";
        self.output += "    fillcolor = \"#ababab\";\n";

        for code_id in ir_fn.codes.iter() {
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
            self.output += &format!("    \"{}{:?}\"[label=\"{}\", shape=\"{}\"]\n", ir_fn.name, code_id, code.dump(ir_fn.value_allocator.clone()), shape);

            // エッジ定義
            if let Some(graph) = local_cfg.get(ir_fn_id).unwrap().successors.get(code_id) {
                for succ_edge in graph.iter() {
                    self.output += &format!("    \"{}{:?}\" -> \"{}{:?}\"\n", ir_fn.name, code_id, ir_fn.name, succ_edge);
                }
            }
        }

        self.output += "  }\n";
    }

    fn to_file(&self) {
        file_util::write_program_into(&self.file_path, self.output.to_string());
    }
}