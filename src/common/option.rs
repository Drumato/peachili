#[derive(Clone)]
pub struct BuildOption {
    pub debug: bool,
    pub verbose: bool,
    pub stop_assemble: bool,
    pub stop_link: bool,
    pub language: Language,
}

impl BuildOption {
    pub fn new(
        debug: bool,
        verbose: bool,
        stop_assemble: bool,
        stop_link: bool,
        language: Language,
    ) -> Self {
        Self {
            debug,
            verbose,
            stop_assemble,
            stop_link,
            language,
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
