#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
    pub stop_assemble: bool,
    pub stop_link: bool,
    pub language: Language,
    pub target: Target,
    pub arch: Architecture,
}

impl Default for BuildOption {
    fn default() -> Self {
        Self {
            debug: false,
            verbose: false,
            stop_assemble: false,
            stop_link: false,
            language: Language::ENGLISH,
            target: Target::X86_64,
            arch: Architecture::X86_64,
        }
    }
}

#[derive(Clone)]
pub enum Language {
    JAPANESE,
    ENGLISH,
}

impl Language {
    pub fn new(env_string: String) -> Self {
        if env_string.contains("ja_JP") {
            return Self::JAPANESE;
        }

        if env_string.as_str() == "C" {
            return Self::ENGLISH;
        }

        panic!("not supported LANG={}", env_string)
    }
}

#[derive(Clone)]
pub enum Target {
    X86_64,
    LLVMIR,
}

impl Target {
    pub fn new(target_str: &str) -> Self {
        match target_str {
            "x86_64" => Self::X86_64,
            "llvm-ir" => Self::LLVMIR,
            _ => panic!("unsupported target -> {}", target_str),
        }
    }
}

#[derive(Clone)]
pub enum Architecture {
    X86_64,
    // ARMV8,
}

impl Architecture {
    pub fn new(arch_str: &str) -> Self {
        match arch_str {
            "x86_64" => Self::X86_64,
            _ => panic!("unsupported architecture -> {}", arch_str),
        }
    }
}
