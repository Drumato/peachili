use std::collections::BTreeMap;
use std::time;

use crate::common::option as opt;
use crate::compiler::general::resource as res;

pub fn resolve_unknown_type_phase(
    _build_option: &opt::BuildOption,
    func_map: &mut BTreeMap<String, res::PFunction>,
    tld_map: &BTreeMap<String, res::TopLevelDecl>,
) {
    let function_number = func_map.len() as u64;
    let resolve_type_pb = indicatif::ProgressBar::new(function_number);
    resolve_type_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    for (func_name, func) in func_map.iter_mut() {
        resolve_type_pb.set_message(&format!("resolve unknown types in {}", func_name));
        func.resolve_type(tld_map);

        resolve_type_pb.inc(1);
    }

    let end = time::Instant::now();

    resolve_type_pb.finish_with_message(&format!(
        "resolve unknown types done!(in {:?})",
        end - start
    ));
}

impl res::PFunction {
    fn resolve_type(&mut self, tld_map: &BTreeMap<String, res::TopLevelDecl>) {
        for (_name, pvar) in self.locals.iter_mut() {
            let current_type = pvar.get_type();

            if let res::PTypeKind::UNRESOLVED(type_name) = &current_type.kind {
                let type_last = res::IdentName::last_name(type_name);
                if tld_map.get(&type_last).is_none() {
                    panic!("should emit a compile-error");
                }

                let resolved_type = tld_map.get(&type_last).unwrap().get_src_type();
                pvar.set_type(resolved_type.clone());
            }
        }
    }
}
