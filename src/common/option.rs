#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
    pub stop_assemble: bool,
    pub stop_link: bool,
    pub language: Language,
}

impl Default for BuildOption {
    fn default() -> Self {
        Self {
            debug: false,
            verbose: false,
            stop_assemble: false,
            stop_link: false,
            language: Language::ENGLISH,
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
