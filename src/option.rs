use structopt::StructOpt;
#[derive(StructOpt, Clone)]
pub struct BuildOption {
    #[structopt(short, long, parse(from_str), default_value = "x86_64")]
    pub target: Target,

    #[structopt(subcommand)]
    pub cmd: Command,

    #[structopt(short, long)]
    pub dump_ir: bool,
}
#[derive(StructOpt, Clone)]
pub enum Command {
    Build,
    Compile { source_file: String },
}

#[derive(Copy, Clone)]
pub enum Target {
    X86_64,
    AArch64,
}

impl From<&str> for Target {
    fn from(s: &str) -> Self {
        match s {
            "x86_64" => Target::X86_64,
            "aarch64" => Target::AArch64,
            _ => panic!("unsupported target -> {}", s),
        }
    }
}
