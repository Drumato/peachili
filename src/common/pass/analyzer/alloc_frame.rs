use crate::common::{tld, peachili_type, frame_object};
use std::collections::BTreeMap;

pub fn allocate_stack_frame(
    _tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
) -> frame_object::StackFrame {
    let mut stack_frame = BTreeMap::new();

    for (scope_name, func_env) in type_env {
        let mut frame_in_func = BTreeMap::new();

        let mut total_offset_in_func = 0;
        // 先に関数以外をすべて回したあと，関数を回す
        // すべてのローカル変数のサイズを合計しないといけないため
        for (entry_name, entry) in func_env {
            if entry.is_function() {
                continue;
            }
            total_offset_in_func += entry.size;
            frame_in_func.insert(entry_name.to_string(), frame_object::FrameObject {
                offset: total_offset_in_func,
            });
        }

        // 先に関数以外をすべて回したあと，関数を回す
        // すべてのローカル変数のサイズを合計しないといけないため
        for (entry_name, entry) in func_env {
            if !entry.is_function() {
                continue;
            }
            total_offset_in_func += entry.size;
            frame_in_func.insert(entry_name.to_string(), frame_object::FrameObject {
                offset: total_offset_in_func,
            });
        }

        stack_frame.insert(scope_name.clone(), frame_in_func);
    }

    stack_frame
}