use crate::common::{module, option};
use crate::compiler::resource as res;

pub fn parse(
    opt: &option::BuildOption,
    tokens: Vec<res::Token>,
    this_mod: module::Module<res::PFunction>,
) -> (module::Module<res::PFunction>, Vec<res::PFunction>) {
    (this_mod, Vec::new())
}
