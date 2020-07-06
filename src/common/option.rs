use clap::ArgMatches;

#[derive(Clone)]
pub struct BuildOption {
    pub matches: ArgMatches,
    pub target: Target,
}

impl BuildOption {
    pub fn new(matches: ArgMatches) -> Self {
        Self {
            matches,
            target: Target::X86_64,
        }
    }

    pub fn get_source(&self) -> String {
        match self.matches.value_of("source") {
            Some(file_path) => file_path.to_string(),
            None => panic!("source file must be specified"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Target {
    X86_64,
}

impl Target {
    pub fn new(target_str: &str) -> Self {
        match target_str {
            "x86_64" => Target::X86_64,
            _ => panic!("unsupported target -> {}", target_str),
        }
    }
}
