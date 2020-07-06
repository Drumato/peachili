#[cfg(test)]
mod module_tests {
    use crate::common::module::*;
    use id_arena::Arena;

    #[test]
    fn module_refs_increment_test() {
        let parent = Module::new_primary(String::new(), String::new());
        let mut arena: Arena<Module> = Arena::new();

        assert_eq!(0, parent.ref_count());

        // 適当にrefを増やす
        for i in 0..3 {
            let e_m = arena.alloc(Module::new_external(String::new(), i.to_string()));
            parent.refs.lock().unwrap().push(e_m);
        }

        assert_eq!(3, parent.ref_count());
    }
}
