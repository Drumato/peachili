#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
    pub stop_assemble: bool,
    pub stop_link: bool,
}

impl BuildOption {
    pub fn new(debug: bool, verbose: bool, stop_assemble: bool, stop_link: bool) -> Self {
        Self {
            debug,
            verbose,
            stop_assemble,
            stop_link,
        }
    }
}
