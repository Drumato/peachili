#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
}

impl BuildOption {
    pub fn new(debug: bool, verbose: bool) -> Self {
        Self { debug, verbose }
    }
}
