use std::{collections::HashMap, rc::Rc};

use crate::compiler::common::frontend::frame_object;

#[derive(Clone, Debug)]
pub struct Scope {
    pub local_variables: HashMap<String, frame_object::FrameObject>,
    pub outer: Option<Rc<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            local_variables: Default::default(),
            outer: None,
        }
    }
}

impl Scope {
    pub fn new(outer: Rc<Self>) -> Self {
        Self {
            local_variables: Default::default(),
            outer: Some(outer),
        }
    }

    /// スコープをさかのぼりながら探索
    /// スコープにエントリが存在するかどうかを検査するのはScopeの責務ではないので，気にしない
    pub fn find_local_var(&self, name: &str) -> frame_object::FrameObject {
        if let Some(v) = self.local_variables.get(name) {
            return v.clone();
        }

        if self.outer.is_none() {
            panic!("cannot find local variable => '{}'", name)
        }

        self.outer.as_ref().unwrap().find_local_var(name)
    }
}
