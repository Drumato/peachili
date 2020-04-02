pub use primary::*;

pub mod primary;

pub trait Module: Sized {
    fn mod_type(&self) -> ModuleType;
    fn is_visited(&self) -> bool;
    fn mod_path(&self) -> &String;
    // fn mod_requires(&self) -> &Vec<ExternalModule>;
}

#[allow(unused)]
pub enum ModuleType {
    PRIMARY,
    EXTERNAL,
}
