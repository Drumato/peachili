mod type_resolve;

pub use type_resolve::*;

mod type_check;

pub use type_check::*;

mod alloc_frame;
pub use alloc_frame::*;

mod constant_folding;
pub use constant_folding::*;
