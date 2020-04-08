#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
    pub stop_assemble: bool,
}

impl BuildOption {
    pub fn new(debug: bool, verbose: bool, stop_assemble: bool) -> Self {
        Self {
            debug,
            verbose,
            stop_assemble,
        }
    }
}
