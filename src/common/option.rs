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
        match self.matches.subcommand() {
            ("build", Some(build_m)) => match build_m.value_of("source") {
                Some(s) => s.to_string(),
                None => panic!("source file must be specified"),
            },
            ("compile", Some(compile_m)) => match compile_m.value_of("source") {
                Some(s) => s.to_string(),
                None => panic!("source file must be specified"),
            },
            _ => panic!("source file must be specified"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Target {
    X86_64,
    AARCH64,
}

impl Target {
    pub fn new(target_str: &str) -> Self {
        match target_str {
            "x86_64" => Target::X86_64,
            "aarch64" => Target::AARCH64,
            _ => panic!("unsupported target -> {}", target_str),
        }
    }
}
