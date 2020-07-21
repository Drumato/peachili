use crate::common::three_address_code as tac;
use crate::common::file_util;

/// 三番地コードの関係をDOT言語で記述し，ファイルに書き出す
pub fn dump_hir(ir_module: &tac::IRModule) {
    let mut dumper = IRDumper {
        output: Default::default(),
        filename: "hir.dot".to_string(),
    };

    dumper.construct_dot_program(ir_module);

    dumper.write_dot_program();
}

struct IRDumper {
    output: String,
    filename: String,
}

impl IRDumper {
    fn construct_dot_program(&mut self, ir_module: &tac::IRModule) {
        self.output += "digraph hir_graph {\n";

        for fn_id in ir_module.funcs.iter() {
            let func = ir_module.fn_allocator.get(*fn_id).unwrap();
            self.append_cluster_function(func);
        }

        self.output += "}\n";
    }

    fn append_cluster_function(&mut self, func: &tac::IRFunction) {
        self.output += &format!("  subgraph cluster_{} {{\n", func.name);
        self.output += &format!("    label = \"{}\";\n", func.name);
        self.output += "    labelloc = \"t\"\n";
        self.output += "    labeljust = \"l\"\n";
        self.output += "    fillcolor = \"#ababab\";\n";

        for code_id in func.codes.iter() {
            let code = func.code_allocator.lock().unwrap().get(*code_id).unwrap().clone();
            self.output += &format!("    {}\n", code.dump(func.value_allocator.clone()));
        }
        self.output += "  }\n";
    }

    fn write_dot_program(self) {
        file_util::write_program_into(&self.filename, self.output)
    }
}