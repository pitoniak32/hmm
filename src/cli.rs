use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Parser, Debug)]
#[command(author, about)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[clap(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    /// command to manage custom man entries for
    pub cmd: Option<String>,

    /// provide this flag to interactively select from available command man entries
    #[arg(short, long)]
    pub interactive: bool,

    /// edit the selected commands man entry
    #[arg(short, long)]
    pub edit: bool,

    /// should run a health check to see status of installed cli tools
    #[arg(long)]
    pub health_check: bool,
}
