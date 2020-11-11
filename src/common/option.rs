use structopt::StructOpt;
#[derive(StructOpt, Clone)]
pub struct BuildOption {
    pub source_file: String,

    #[structopt(parse(from_str), default_value = "x86_64")]
    pub target: Target,

    #[structopt(subcommand)]
    pub cmd: Command,
}


#[derive(StructOpt, Clone)]
pub enum Command {
    Build,
    Compile,
}


#[derive(Copy, Clone)]
pub enum Target {
    X86_64,
}

impl From<&str> for Target{
    fn from(s: &str) -> Self {
        match s {
            "x86_64" => Target::X86_64,
            _ => panic!("unsupported target -> {}", s),
        }
    }
}
