#[cfg(test)]
mod module_tests {
    use crate::common::module::*;
    use typed_arena::Arena;

    #[test]
    fn module_refs_increment_test() {
        let parent = ModuleData::new_primary(String::new(), String::new());
        let arena = Arena::new();

        assert_eq!(0, parent.ref_count());

        // 適当にrefを増やす
        for i in 0..3 {
            let e_m = arena.alloc(ModuleData::new_external(String::new(), i.to_string()));
            parent.refs.lock().unwrap().push(e_m);
        }

        assert_eq!(3, parent.ref_count());
    }
}